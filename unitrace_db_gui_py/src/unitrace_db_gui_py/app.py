"""Tkinter GUI murni (default look)."""

from __future__ import annotations

import tkinter as tk
from tkinter import messagebox, ttk

from .ffi import UnitraceFFI, open_default


class UnitraceApp:
    def __init__(self, root: tk.Tk, ffi: UnitraceFFI, column_names: list[str]) -> None:
        self.root = root
        self.ffi = ffi
        self.column_names = column_names
        self._build_ui()

    def _build_ui(self) -> None:
        self.root.title("Unitrace - Database Search")
        self.root.geometry("1100x600")

        top = tk.Frame(self.root)
        top.pack(fill=tk.X, padx=10, pady=10)

        tk.Label(top, text="Cari:").pack(side=tk.LEFT)

        self.entry = tk.Entry(top, width=80)
        self.entry.pack(side=tk.LEFT, padx=5)
        self.entry.focus_set()
        self.entry.bind("<Return>", self._on_search)

        tk.Button(top, text="Cari", command=self._on_search).pack(side=tk.LEFT)

        self.status_var = tk.StringVar(value="Siap mencari.")
        tk.Label(self.root, textvariable=self.status_var).pack(
            fill=tk.X, padx=10, anchor="w"
        )

        self.not_found_frame = tk.Frame(self.root)
        self.not_found_var = tk.StringVar()
        tk.Label(
            self.not_found_frame, textvariable=self.not_found_var, justify=tk.LEFT
        ).pack()

        table_frame = tk.Frame(self.root)
        table_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)

        self.tree = ttk.Treeview(table_frame, show="headings")
        self.tree.pack(fill=tk.BOTH, expand=True)

        # Setup kolom sekali di awal
        cols = [f"col{i}" for i in range(len(self.column_names))]
        self.tree["columns"] = cols
        for i, name in enumerate(self.column_names):
            self.tree.heading(cols[i], text=name.upper())
            self.tree.column(cols[i], width=130, anchor="w")

    def _on_search(self, _event: tk.Event | None = None) -> None:
        keyword = self.entry.get().strip()
        if not keyword:
            return

        result = self.ffi.search(keyword)
        if result is None:
            self.status_var.set("ERROR: FFI returned null")
            return

        for row in self.tree.get_children():
            self.tree.delete(row)
        self.not_found_frame.pack_forget()
        self.status_var.set(result["status_text"])

        if proof := result.get("not_found_proof"):
            self.not_found_var.set(
                f"DATA TIDAK DITEMUKAN\n"
                f"Kata kunci: {proof['keyword']}\n"
                f"Cakupan: {proof['searched_columns']} kolom x {proof['searched_rows']} baris\n"
                f"Status DB: {proof['database_version']}"
            )
            self.not_found_frame.pack(fill=tk.X, padx=10, pady=5)
        elif result["rows"]:
            for row in result["rows"]:
                self.tree.insert("", tk.END, values=row)


def launch() -> None:
    ffi, init_result = open_default()

    root = tk.Tk()
    if init_result["status"] != "ok":
        root.withdraw()
        messagebox.showerror("Database Error", init_result["message"])
        return

    UnitraceApp(root, ffi, init_result["columns"])
    root.mainloop()
