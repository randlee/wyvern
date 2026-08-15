(function() {
  let allRows = [];
  let columns = [];
  let sortState = { col: null, asc: true };
  let colFilters = [];

  async function init() {
    const resp = await fetch('../data/rows.json');
    const data = await resp.json();
    columns = data.columns;
    allRows = data.rows;

    if (data.meta && data.meta.truncated) {
      const banner = document.getElementById('truncation-banner');
      banner.hidden = false;
      document.getElementById('truncation-msg').textContent =
        `Showing first ${data.meta.row_count.toLocaleString()} rows (file truncated).`;
    }

    colFilters = Array(columns.length).fill('');
    buildTable();
    setupSearch();
  }

  function buildTable() {
    const container = document.getElementById('table-container');
    container.innerHTML = '';
    const table = document.createElement('table');
    table.id = 'csv-table';

    // Header
    const thead = table.createTHead();
    const headerRow = thead.insertRow();
    columns.forEach((col, i) => {
      const th = document.createElement('th');
      th.className = 'sortable';
      th.dataset.col = i;
      th.textContent = col;
      const indicator = document.createElement('span');
      indicator.className = 'sort-indicator';
      th.appendChild(indicator);
      th.addEventListener('click', () => toggleSort(i));
      headerRow.appendChild(th);
    });

    // Filter row
    const filterRow = thead.insertRow();
    colFilters = columns.map((_, i) => {
      const td = filterRow.insertCell();
      const input = document.createElement('input');
      input.type = 'text';
      input.placeholder = '…';
      input.className = 'col-filter';
      input.dataset.col = i;
      let debounce;
      input.addEventListener('input', () => {
        clearTimeout(debounce);
        debounce = setTimeout(() => { colFilters[i] = input.value.toLowerCase(); applyFilters(); }, 200);
      });
      td.appendChild(input);
      return '';
    });

    // Body
    const tbody = table.createTBody();
    tbody.id = 'csv-body';
    container.appendChild(table);

    applyFilters();

    // Finish button
    const finishBtn = document.createElement('button');
    finishBtn.id = 'finish-btn';
    finishBtn.textContent = 'Finish';
    finishBtn.addEventListener('click', () => {
      if (window.wyvern) {
        window.wyvern.finish({ button: 'finish', data: { row_count: allRows.length }, stack: [] });
      }
    });
    container.appendChild(finishBtn);
  }

  function getVisible() {
    const globalSearch = document.getElementById('global-search').value.toLowerCase();
    return allRows.filter(row => {
      const matchesGlobal = !globalSearch || row.some(cell => String(cell).toLowerCase().includes(globalSearch));
      const matchesCols = colFilters.every((f, i) => !f || String(row[i] ?? '').toLowerCase().includes(f));
      return matchesGlobal && matchesCols;
    });
  }

  function applyFilters() {
    const visible = sortRows(getVisible());
    const tbody = document.getElementById('csv-body');
    if (!tbody) return;
    tbody.innerHTML = '';
    visible.forEach(row => {
      const tr = tbody.insertRow();
      row.forEach(cell => { tr.insertCell().textContent = cell ?? ''; });
    });
  }

  function sortRows(rows) {
    if (sortState.col === null) return rows;
    const col = sortState.col;
    const asc = sortState.asc;
    return [...rows].sort((a, b) => {
      const av = a[col] ?? '', bv = b[col] ?? '';
      const an = parseFloat(av), bn = parseFloat(bv);
      if (!isNaN(an) && !isNaN(bn)) return asc ? an - bn : bn - an;
      return asc ? String(av).localeCompare(String(bv)) : String(bv).localeCompare(String(av));
    });
  }

  function toggleSort(col) {
    if (sortState.col === col) { sortState.asc = !sortState.asc; }
    else { sortState.col = col; sortState.asc = true; }
    document.querySelectorAll('.sort-indicator').forEach((el, i) => {
      el.textContent = i === col ? (sortState.asc ? ' ▲' : ' ▼') : '';
    });
    applyFilters();
  }

  function setupSearch() {
    const input = document.getElementById('global-search');
    let debounce;
    input.addEventListener('input', () => { clearTimeout(debounce); debounce = setTimeout(applyFilters, 200); });
  }

  document.addEventListener('DOMContentLoaded', init);
})();
