# GitHub Codespace Orchestrator

Automated multi-account orchestrator untuk deploy dan manage Mawari & Nexus nodes di GitHub Codespaces.

## Features

- **Multi-account rotation**: Automatic token rotation setiap 20 jam
- **Nuke & create strategy**: Clean deployment setiap cycle
- **State persistence**: Resume dari token terakhir jika restart
- **Auto-verification**: Check codespace status after creation
- **Error handling**: Skip invalid tokens, retry on failure

---

## Requirements

### 1. Rust Toolchain

**Windows:**

Download dan install dari: https://rustup.rs/

Atau via PowerShell:
```powershell
# Download installer
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"

# Run installer
.\rustup-init.exe
```

Pilih default installation (option 1).

**Verify installation:**
```powershell
rustc --version
cargo --version
```

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. C/C++ Build Tools

**Windows:**

Download dan install **Visual Studio 2022 Build Tools**: https://visualstudio.microsoft.com/downloads/

Pilih workload: **Desktop development with C++**

Minimal components yang dibutuhkan:
- MSVC v143 - VS 2022 C++ x64/x86 build tools
- Windows 10/11 SDK

Atau via command line:
```powershell
# Download installer
Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile "vs_buildtools.exe"

# Install minimal components
.\vs_buildtools.exe --quiet --wait --norestart --nocache `
  --installPath C:\BuildTools `
  --add Microsoft.VisualStudio.Workload.VCTools `
  --add Microsoft.VisualStudio.Component.Windows11SDK.22000
```

**Linux:**
```bash
# Debian/Ubuntu
sudo apt-get update
sudo apt-get install build-essential

# RHEL/CentOS/Fedora
sudo yum groupinstall "Development Tools"
```

**macOS:**
```bash
xcode-select --install
```

### 3. GitHub CLI

**Windows:**

Download installer: https://cli.github.com/

Atau via winget:
```powershell
winget install --id GitHub.cli
```

**Linux:**
```bash
# Debian/Ubuntu
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update
sudo apt install gh

# Fedora/RHEL/CentOS
sudo dnf install gh
```

**macOS:**
```bash
brew install gh
```

**Verify installation:**
```bash
gh --version
```

---

## Setup

### 1. Clone Repository

```bash
git clone https://github.com/Kyugito666/codespace-orchestrator.git
cd codespace-orchestrator
```

### 2. Configure Tokens

Copy template:
```bash
# Windows
copy tokens.json.example tokens.json

# Linux/macOS
cp tokens.json.example tokens.json
```

Edit `tokens.json`:
```json
{
  "tokens": [
    "ghp_YourFirstTokenHere",
    "ghp_YourSecondTokenHere",
    "ghp_YourThirdTokenHere"
  ]
}
```

**Cara generate GitHub token:**

1. Login ke GitHub account
2. Buka: https://github.com/settings/tokens
3. Click **"Generate new token (classic)"**
4. Set permissions:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `codespace` (Full control of codespaces)
5. Set expiration: **90 days** (maximum)
6. Click **"Generate token"**
7. Copy token (format: `ghp_...`)
8. Ulangi untuk akun lain jika punya multiple accounts

**Tips:**
- Gunakan multiple GitHub accounts untuk maximize free tier
- Setiap akun free tier dapat ~60 hours/month codespace usage
- Generate token untuk semua akun, masukkan ke `tokens.json`

### 3. Build Orchestrator

```bash
cargo build --release
```

Build pertama akan lama (5-10 menit) karena download dependencies.

**Expected output:**
```
   Compiling serde v1.0.x
   Compiling serde_json v1.0.x
   Compiling orchestrator v0.3.0
    Finished release [optimized] target(s) in XX.XXs
```

### 4. Setup Runtime Repository

Fork atau clone runtime repository:
```bash
# Fork di GitHub UI, atau clone:
git clone https://github.com/Kyugito666/mawari-nexus-blueprint.git
```

Set **Codespace Secrets** di repository settings:

Buka: `https://github.com/YOUR_USERNAME/mawari-nexus-blueprint/settings/secrets/codespaces`

Tambahkan 5 secrets:

| Secret Name | Value | Keterangan |
|-------------|-------|------------|
| `MAWARI_OWNER_ADDRESS` | `0xYourAddress...` | Main wallet address |
| `MAWARI_BURNER_ADDRESS` | `0xBurnerAddress...` | Burner wallet address |
| `MAWARI_BURNER_PRIVATE_KEY` | `0x1234...` | Burner private key |
| `NEXUS_WALLET_ADDRESS` | `0xYourAddress...` | Main wallet address |
| `NEXUS_NODE_ID` | `12345678` | From https://app.nexus.xyz/nodes |

**Important:**
- **JANGAN** pakai main wallet untuk burner!
- Generate wallet baru khusus untuk Mawari burner
- Burner wallet dipakai untuk transaction fees

---

## Usage

### Basic Commands

**Run orchestrator:**
```bash
# Windows
cargo run --release -- YOUR_USERNAME/mawari-nexus-blueprint

# Linux/macOS
cargo run --release -- YOUR_USERNAME/mawari-nexus-blueprint
```

**Check status:**
```bash
cargo run --release -- status
```

**Verify current nodes:**
```bash
cargo run --release -- verify
```

### First Run

```bash
cargo run --release -- Kyugito666/mawari-nexus-blueprint
```

**Expected output:**
```
==================================================
   ORCHESTRATOR - NUKE & CREATE STRATEGY
==================================================

Loading tokens.json...
Loaded 37 tokens
Target Repo: Kyugito666/mawari-nexus-blueprint

Starting loop...

--------------------------------------------------
Token #1 of 37
Valid token for: @username1

Deploying for @username1...
  Scanning existing codespaces...
  No old codespaces found
  Creating new codespaces...
    [1/2] Creating mawari-node (basicLinux32gb)...
       Mawari: mawari-node-xxxxx
    [2/2] Creating nexus-node (standardLinux32gb)...
       Nexus: nexus-node-yyyyy
  Verifying deployment...
   Waiting for 'mawari-node-xxxxx' to be ready...
      Checking... (1/3)
      Not ready, waiting 20s...
      Checking... (2/3)
   Codespace Available! (Auto-start will run)
   Waiting for 'nexus-node-yyyyy' to be ready...
      Checking... (1/3)
   Codespace Available! (Auto-start will run)

==================================================
         DEPLOYMENT SUCCESS
==================================================
Account  : @username1
Mawari   : mawari-node-xxxxx
Nexus    : nexus-node-yyyyy
State saved

Running for 20 hours...
Sleeping...
```

Program akan running selama 20 jam, lalu otomatis ganti ke token berikutnya.

### Status Checking

**Check orchestrator state:**
```bash
cargo run --release -- status
```

Output:
```
STATUS ORCHESTRATOR
==========================================
State file found
Current Token Index: 0
Mawari Node: mawari-node-xxxxx
Nexus Node: nexus-node-yyyyy

Tokens Available:
   Total: 37 tokens
```

**Verify nodes are running:**
```bash
cargo run --release -- verify
```

Output:
```
VERIFIKASI NODE AKTIF
==========================================
Token Index: 0

Verifying Mawari: mawari-node-xxxxx
   RUNNING & READY

Verifying Nexus: nexus-node-yyyyy
   RUNNING & READY
```

---

## How It Works

### Nuke & Create Strategy

1. **Scan**: Check existing codespaces di repository
2. **Cleanup**: Stop dan delete semua codespace lama
3. **Create**: Buat 2 codespace baru:
   - `mawari-node` dengan `basicLinux32gb` (2-core, 8GB RAM)
   - `nexus-node` dengan `standardLinux32gb` (4-core, 16GB RAM)
4. **Verify**: Wait sampai state = "Available"
5. **Monitor**: Running selama 20 jam
6. **Rotate**: Ganti ke token berikutnya
7. **Repeat**: Loop kembali ke step 1

### Why 20 Hours?

- GitHub Free tier: ~60 hours/month per account
- 20 jam × 3 cycles = 60 jam
- Optimal untuk maximize usage tanpa exceed quota
- Auto-cleanup sebelum billing limit

### State Persistence

File `state.json` menyimpan:
```json
{
  "current_account_index": 0,
  "current_mawari_name": "mawari-node-xxxxx",
  "current_nexus_name": "nexus-node-yyyyy"
}
```

Jika orchestrator di-restart, akan melanjutkan dari token terakhir.

---

## Monitoring

### Via GitHub CLI

```bash
# List all codespaces
gh codespace list

# View specific codespace
gh codespace view mawari-node-xxxxx

# SSH into codespace
gh codespace ssh -c mawari-node-xxxxx
```

### Inside Codespace

**Mawari (Docker):**
```bash
# Check container
docker ps

# View logs
docker logs -f mawari-node

# Check burner wallet
cat ~/mawari/mawari_data/flohive-cache.json
```

**Nexus (Tmux):**
```bash
# List sessions
tmux ls

# Attach to session
tmux attach -t nexus

# Detach: Ctrl+B then D

# Check status
nexus-cli status
```

### Via Web

1. Buka: https://github.com/codespaces
2. Click codespace name
3. Otomatis buka VS Code di browser
4. Buka terminal, jalankan command monitoring

---

## Troubleshooting

### Build Errors

**Error: "linker not found"**

Install C++ build tools (lihat Requirements).

**Error: "failed to run custom build command"**

```bash
# Update Rust
rustup update

# Clean build cache
cargo clean
cargo build --release
```

### Token Errors

**Error: "Bad credentials"**

Token expired atau invalid. Generate token baru:
1. Buka https://github.com/settings/tokens
2. Generate new token
3. Update `tokens.json`

**Error: "insufficient quota"**

Akun sudah exceed free tier. Orchestrator akan skip token ini.

### Codespace Errors

**Error: "Failed to create codespace"**

Possible causes:
- Repository tidak ada atau private
- Token tidak punya permission
- Machine type tidak tersedia

Check manual:
```bash
gh codespace create -r YOUR_USERNAME/mawari-nexus-blueprint -m basicLinux32gb
```

**Node tidak jalan di codespace**

SSH masuk dan check logs:
```bash
gh codespace ssh -c mawari-node-xxxxx

# Check setup log
cat /workspaces/mawari-nexus-blueprint/setup.log
cat /workspaces/mawari-nexus-blueprint/autostart.log

# Manual restart
bash /workspaces/mawari-nexus-blueprint/auto-start.sh
```

### State Issues

**Reset state:**
```bash
# Windows
del state.json

# Linux/macOS
rm state.json
```

Orchestrator akan mulai dari token pertama.

---

## Advanced Usage

### Custom Run Duration

Edit `main.rs`:
```rust
const RUN_DURATION: Duration = Duration::from_secs(10 * 3600); // 10 jam
```

Rebuild:
```bash
cargo build --release
```

### Selective Token Usage

Edit `tokens.json` untuk hanya include tokens yang ingin dipakai:
```json
{
  "tokens": [
    "ghp_Token1",
    "ghp_Token3"
  ]
}
```

Token 2 akan di-skip.

### Manual Cleanup

Delete specific codespace:
```bash
gh codespace delete mawari-node-xxxxx --force
```

Delete all codespaces in repo:
```bash
gh codespace list -r YOUR_USERNAME/mawari-nexus-blueprint --json name -q ".[].name" | ForEach-Object { gh codespace delete $_ --force }
```

---

## File Structure

```
codespace-orchestrator/
├── src/
│   ├── main.rs              # Entry point & main loop
│   ├── config.rs            # Config & state management
│   └── github.rs            # GitHub CLI wrapper
├── Cargo.toml               # Dependencies
├── .gitignore               # Ignore tokens & state
├── tokens.json.example      # Template
├── tokens.json              # Your tokens (git-ignored)
├── state.json               # Auto-generated (git-ignored)
└── README.md                # This file
```

---

## Security

- `tokens.json` sudah di `.gitignore` - **JANGAN commit!**
- `state.json` juga git-ignored
- Secrets disimpan di GitHub Codespace Secrets (encrypted)
- Token auto-expire setelah 90 hari
- Gunakan burner wallet untuk Mawari (JANGAN main wallet)

---

## Contributing

Issues dan Pull Requests welcome di: https://github.com/Kyugito666/codespace-orchestrator

---

## License

MIT License - Free to use and modify

---

## Related Projects

- **Runtime Repository**: https://github.com/Kyugito666/mawari-nexus-blueprint
- **Mawari Network**: https://mawari.network/
- **Nexus**: https://nexus.xyz/

---

## FAQ

**Q: Berapa banyak akun yang dibutuhkan?**

Minimal 1, recommended 3-5 untuk continuous uptime.

**Q: Apakah aman untuk codespace idle 4 jam?**

Ya, orchestrator running 20 jam untuk buffer sebelum billing. Codespace akan auto-restart jika idle.

**Q: Bisa running 24/7?**

Ya dengan multiple accounts. 60 hours/month × 3 accounts = 180 hours = 6 hari non-stop.

**Q: Token bisa dishare?**

TIDAK! Token = full access ke account. Generate sendiri untuk setiap akun.

**Q: Codespace dihapus tiap cycle?**

Ya, nuke & create strategy untuk clean state. Data tidak persistent.

**Q: Burner wallet harus isi balance?**

Tidak perlu untuk testnet. Tapi recommended minimal gas fee jika mainnet.

---

**Ready to deploy! 🚀**
