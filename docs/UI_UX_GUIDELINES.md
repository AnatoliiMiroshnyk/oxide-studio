# UI and interaction guidelines

## 1. Experience principles

- **Workbench, not wizard:** keep the object explorer, editor, result surface, and status information close together.
- **Progressive disclosure:** reveal detailed metadata and advanced actions only in the relevant workspace.
- **Clear state:** always show the active database, read-only state, current selection, and last operation outcome.
- **Safe by default:** never make a destructive action look like a routine navigation action; show affected scope before applying it.
- **Keyboard-friendly:** every primary workflow must be reachable without a mouse.

## 2. Information architecture

The primary application frame should remain stable:

1. Top application bar — product identity, active connection, open/create controls, run action, global actions.
2. Left explorer — searchable database objects grouped by tables, views, indexes, and triggers; connection and workspace navigation.
3. Main workspace — contextual data browser, structure inspector, or SQL editor/result tabs.
4. Bottom status strip — operation state, error summary, SQLite version, and read-only/local-session indicators.

Future advanced workspaces may add import/export, diagnostics, ER diagrams, comparison, and backup/restore without changing the primary navigation model.

## 3. Visual system

The v0.1.0 theme defines a dark navy surface with a restrained teal accent. Keep semantic colors consistent:

| Token | Purpose | Current direction |
|---|---|---|
| `surface.canvas` | Main window background | Deep navy |
| `surface.panel` | Explorer and toolbar surfaces | Dark blue-grey |
| `surface.elevated` | Dialogs, cards, result frames | Slightly lighter blue-grey |
| `text.primary` | Main content | High-contrast cool white |
| `text.secondary` | Metadata and helper copy | Muted blue-grey |
| `accent.primary` | Selection, primary action, focus | Teal |
| `state.success` | Successful operation | Green/teal |
| `state.warning` | Read-only, truncation, caution | Amber |
| `state.error` | Failed operation | Red |

Use spacing in a consistent 4-point rhythm. Favor clear grouping, visible labels, and readable monospace typography for SQL, identifiers, and raw values.

## 4. Interaction rules

- Opening a database must never silently create a new file; create is a separate explicit flow.
- Read-only state must be visible in the header and reflected in disabled write controls.
- Long-running tasks must show progress and offer cancellation before production use.
- Destructive operations must show object/table scope, a summary of affected rows/objects where possible, and a confirmation step.
- Query errors should preserve the editor contents and identify the statement location when available.
- Result truncation must be visually explicit and explain how to export the full result.
- Empty states should explain the next useful action rather than displaying a blank panel.
- Avoid modal dialogs for routine navigation; reserve them for file selection, destructive confirmation, and focused data-entry flows.

## 5. Accessibility and localization

- Preserve keyboard focus indication and logical tab order.
- Do not communicate status with color alone; pair color with text/icon labels.
- Provide accessible names for icon-only buttons and tooltips for unfamiliar actions.
- Respect OS text scaling where practical; avoid fixed-height text containers that clip content.
- Keep user-visible strings centralized before adding Ukrainian and additional translations.
- Test common screen sizes down to the documented minimum window dimensions.

## 6. Prototype deviations

The current UI is a foundation, not the final design system. It has a dark theme only, a minimal shortcut set, no command palette, no accessibility audit, and no persistent layout preferences. The visual tokens above should be moved into typed theme configuration before adding light/high-contrast themes.
