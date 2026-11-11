
from __future__ import annotations

import tkinter as tk
from tkinter import messagebox, simpledialog, ttk

from .config import get_password
from .ffi import UnitraceFFI, open_default


class UnitraceApp:
    def __init__(
        self, root: tk.Tk, ffi: UnitraceFFI, columns: list[str], sig_ok: bool
    ) -> None:
        self.root = root
        self.ffi = ffi
        self.columns = columns
        self.sig_ok = sig_ok
        self._build()

    def _build(self) -> None:
        self.root.title("Unitrace — Dev Inspector")
        self.root.geometry("1100x600")

        top = tk.Frame(self.root)
        top.pack(fill=tk.X, padx=10, pady=10)

        tk.Label(top, text="Cari:").pack(side=tk.LEFT)
        self.entry = tk.Entry(top, width=80)
        self.entry.pack(side=tk.LEFT, padx=5)
        self.entry.focus_set()
        self.entry.bind("<Return>", self._on_search)
        tk.Button(top, text="Cari", command=self._on_search).pack(side=tk.LEFT)

        self.status_var = tk.StringVar(value="Siap.")
        tk.Label(self.root, textvariable=self.status_var).pack(
            fill=tk.X, padx=10, anchor="w"
        )

        badge_text = "SIGNATURE: OK" if self.sig_ok else "SIGNATURE: MISSING"
        badge_bg = "#ccffcc" if self.sig_ok else "#ffcccc"
        tk.Label(
            self.root,
            text=badge_text,
            bg=badge_bg,
            font=("TkDefaultFont", 10, "bold"),
        ).pack(fill=tk.X, padx=10)

        self.nf_frame = tk.Frame(self.root)
        self.nf_var = tk.StringVar()
        tk.Label(self.nf_frame, textvariable=self.nf_var, justify=tk.LEFT).pack()

        table = tk.Frame(self.root)
        table.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        self.tree = ttk.Treeview(table, show="headings")
        self.tree.pack(fill=tk.BOTH, expand=True)

        cols = [f"c{i}" for i in range(len(self.columns))]
        self.tree["columns"] = cols
        for i, name in enumerate(self.columns):
            self.tree.heading(cols[i], text=name.upper())
            self.tree.column(cols[i], width=130, anchor="w")

    def _on_search(self, _ev=None) -> None:
        kw = self.entry.get().strip()
        if not kw:
            return
        result = self.ffi.search(kw)
        if result is None:
            self.status_var.set("ERROR: FFI returned null")
            return

        for row in self.tree.get_children():
            self.tree.delete(row)
        self.nf_frame.pack_forget()
        self.status_var.set(result["status_text"])

        if proof := result.get("not_found_proof"):
            self.nf_var.set(
                f"TIDAK DITEMUKAN\n"
                f"Keyword: {proof['keyword']}\n"
                f"Scope: {proof['searched_columns']} kolom x {proof['searched_rows']} baris\n"
                f"DB version: {proof['database_version']}"
            )
            self.nf_frame.pack(fill=tk.X, padx=10, pady=5)
        elif result["rows"]:
            for row in result["rows"]:
                self.tree.insert("", tk.END, values=row)


def launch() -> None:
    root = tk.Tk()
    root.withdraw()

    password = get_password()
    if not password:
        password = simpledialog.askstring(
            "Unitrace — Dev Inspector",
            "Password database:",
            show="*",
            parent=root,
        )
    if not password:
        root.destroy()
        return

    try:
        ffi, result = open_default(password)
    except FileNotFoundError as e:
        messagebox.showerror("Error", str(e))
        root.destroy()
        return

    if result["status"] != "ok":
        messagebox.showerror("Gagal buka package", result["message"])
        root.destroy()
        return

    root.deiconify()
    UnitraceApp(
        root,
        ffi,
        result["columns"],
        sig_ok=bool(result.get("signature_verified")),
    )
    try:
        root.mainloop()
    finally:
        ffi.close()
