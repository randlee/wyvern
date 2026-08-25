# Agent DAG demo — execution deferral (g.7)

Companion to [g7-dag-agent-execution.md](g7-dag-agent-execution.md). Finish payload, export path, and AC live only in that sprint doc.

Wave 2 **visualizes and exports**. It does not run agents, spawn Task tools, or interpret the DAG in Rust (ADR-0006).

A separate post-publish project consumes `./wyvern-dag-export.json`. Welcome copy must say execution is deferred.

Out of Wyvern: acyclicity checks in Rust, live execution status, spawning Cursor / Claude / ATM agents.
