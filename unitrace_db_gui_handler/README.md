# `unitrace_db_gui_handler` API Reference

Contract-only crate for the unitrace GUI layer. Contains presentation types
(`ViewState`, `RowViewModel`, `NotFoundProof`) and the `GuiController` trait.

No I/O, no framework bindings, no implementation. Downstream crates
(`unitrace_db_gui_core`) implement the trait; the facade `unitrace_db`
re-exports these types for FFI and CLI consumers.
