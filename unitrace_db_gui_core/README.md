# `unitrace_db_gui_core` — implementation

Concrete implementation of `unitrace_db_gui_handler::GuiController` on top of
`unitrace_db_core::searcher::DefaultSearcher`.

Depends on `unitrace_db_core`, `unitrace_db_handler`, and `unitrace_db_gui_handler`.
Intentionally **does not** depend on the `unitrace_db` facade — that would
introduce a cycle, since the facade re-exports `DefaultGuiController` from here.
