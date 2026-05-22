# Fixing `dlltool.exe: program not found` on Windows

## Problem

When building Rust projects (especially those with FFI or C dependencies), you may encounter:

```
Error calling dlltool 'dlltool.exe': program not found
```

This happens because `dlltool.exe` is a utility from **MSYS2 / MinGW** that is used by Rust's build system (e.g., via `cc` or `pkg-config` build scripts) to generate import libraries from DLLs. It is **not** included with Windows, Rust, or Visual Studio by default.

## Solution: Install MSYS2

### 1. Install MSYS2 via winget

Open **PowerShell as Administrator** and run:

```powershell
winget install --id MSYS2.MSYS2 --accept-source-agreements 2>&1
```

This downloads and installs MSYS2 to the default location (`C:\msys64`).

### 2. Add MSYS2 `usr\bin` to your PATH

`dlltool.exe` is located at:

```
C:\msys64\usr\bin\dlltool.exe
```

You need to add `C:\msys64\usr\bin` to your system `PATH` environment variable. (or C:\msys64\mingw64\bin)

#### Option A: Via PowerShell (Admin)

```powershell
[Environment]::SetEnvironmentVariable(
    "Path",
    [Environment]::GetEnvironmentVariable("Path", "Machine") + ";C:\msys64\usr\bin",
    "Machine"
)
```

#### Option B: Via System Settings

1. Press <kbd>Win</kbd> + <kbd>R</kbd>, type `sysdm.cpl`, and press Enter.
2. Go to the **Advanced** tab → **Environment Variables**.
3. Under **System variables**, select `Path` → **Edit**.
4. Click **New** and add: `C:\msys64\usr\bin` or C:\msys64\mingw64\bin
5. Click **OK** on all dialogs.

### 3. Verify the fix

Open a **new** terminal (so the updated PATH takes effect) and run:

```powershell
where dlltool
```

You should see output like:

```
C:\msys64\usr\bin\dlltool.exe
```

Then try your Rust build again:

```powershell
cargo build --workspace
```

## Alternative: Use the MSYS2 Shell

If you prefer not to modify your system PATH, you can run your build commands from the **MSYS2 MinGW64 shell** (installed as part of MSYS2):

1. Launch **MSYS2 MinGW64** from the Start Menu.
2. Navigate to your project directory.
3. Run `cargo build` from that shell.

## Notes

- `dlltool.exe` is part of the **binutils** package, which is included with the base MSYS2 installation. No additional pacman packages are required.
- If you already have MSYS2 installed but `dlltool` is still not found, ensure `C:\msys64\usr\bin` is in your PATH and that you've opened a **new terminal** after adding it.
- This error is most common when building crates that use `cc` or `pkg-config` build scripts (e.g., `ring`, `openssl-sys`, or custom FFI bindings).
