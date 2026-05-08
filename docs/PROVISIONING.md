# Pre-Provisioning Runbook

Steps to perform on a fresh VPS *before* starting the agent. Do these as root or via sudo. The agent never needs to run any of these.

The goal is a host where the `loom` user has everything it needs in userspace and nothing else.

---

## 1. Provision the VPS

Recommended specs: 4 vCPU, 16 GB RAM, 100 GB SSD. Loom Lens is smaller than Loom main; modest specs work. Ubuntu 24.04 LTS or Debian 12.

## 2. Initial hardening

Same as Loom main project. Briefly:

```bash
apt update && apt full-upgrade -y
apt install -y unattended-upgrades crowdsec nftables
dpkg-reconfigure -plow unattended-upgrades

useradd -m -s /bin/bash loom
passwd -l loom

mkdir -p /home/loom/.ssh
chmod 700 /home/loom/.ssh
cat > /home/loom/.ssh/authorized_keys <<EOF
ssh-ed25519 AAAA... your-key-here
EOF
chmod 600 /home/loom/.ssh/authorized_keys
chown -R loom:loom /home/loom/.ssh

cat > /etc/ssh/sshd_config.d/99-loom.conf <<EOF
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
EOF
systemctl restart sshd

systemctl enable --now crowdsec
```

## 3. Install nftables egress allowlist

```bash
mkdir -p /etc/nftables.d

# After the repo is cloned (step 6), copy:
# cp /opt/loom-lens/nftables/egress.nft /etc/nftables.d/loom-lens-egress.nft

cat >> /etc/nftables.conf <<EOF

include "/etc/nftables.d/*.nft"
EOF

systemctl enable --now nftables

# Hourly egress refresh.
cat > /etc/cron.d/loom-lens-egress <<EOF
0 * * * * root /opt/loom-lens/scripts/refresh-egress.sh >/dev/null 2>&1
EOF
```

## 4. Install mise (as the loom user)

```bash
sudo -u loom -i bash -c '
curl https://mise.run | sh
echo "eval \"\$(~/.local/bin/mise activate bash)\"" >> ~/.bashrc
'
```

## 5. Generate the GitHub deploy key

```bash
sudo -u loom -i bash -c '
ssh-keygen -t ed25519 -f ~/.ssh/loom_lens_deploy -N ""
cat ~/.ssh/loom_lens_deploy.pub
'
```

Copy the public key. On `github.com/TheFamousHesham/loom-lens`:
- **Settings → Deploy keys → Add deploy key**
- **Title:** `loom-lens-vps`
- **Allow write access:** ✅ (the agent needs to push)
- Paste the public key.

## 6. Clone the project

```bash
mkdir -p /opt/loom-lens
chown loom:loom /opt/loom-lens

sudo -u loom -i bash -c '
cd /opt/loom-lens
git clone git@github.com:TheFamousHesham/loom-lens.git .
'
```

## 7. Apply the egress allowlist

```bash
cp /opt/loom-lens/nftables/egress.nft /etc/nftables.d/loom-lens-egress.nft
nft -f /etc/nftables.conf
/opt/loom-lens/scripts/refresh-egress.sh
```

## 8. Install the systemd unit

```bash
cp /opt/loom-lens/systemd/loom-lens-agent.service /etc/systemd/system/
systemctl daemon-reload
# Don't enable yet — start manually first to verify.
```

## 9. Sudoers for egress refresh

```bash
cat > /etc/sudoers.d/loom-lens-egress <<EOF
loom ALL=(root) NOPASSWD: /opt/loom-lens/scripts/refresh-egress.sh
EOF
chmod 0440 /etc/sudoers.d/loom-lens-egress
```

## 10. Configure .env and run init

```bash
sudo -u loom -i
cd /opt/loom-lens
cp .env.example .env
chmod 600 .env
$EDITOR .env  # Fill in ANTHROPIC_API_KEY, etc.
./scripts/init.sh
```

If `init.sh` succeeds, the host is ready.

---

## Pre-flight checklist

Before kicking off the agent:

- [ ] `whoami` returns `loom`
- [ ] `sudo -n true` fails (no sudo for loom except refresh-egress.sh)
- [ ] `mise list` shows pinned tools
- [ ] `curl https://example.com` fails or hangs (egress allowlist active)
- [ ] `curl -I https://api.anthropic.com` succeeds
- [ ] `.env` has `chmod 600`
- [ ] `git push` works (deploy key has write access)
- [ ] `STATUS.md`, `BLOCKED.md`, `SECURITY.md`, `PORTABILITY.md` exist
- [ ] **Anthropic API key has a hard spend cap set in the console**

---

## Especially important for a public repo

- [ ] **Verify `.gitignore` includes `.env`** — `git status` should NOT show `.env` as a tracked file
- [ ] **Test the pre-commit hook** — try to commit a fake key; the hook should block it
- [ ] **Verify no secrets in the repo today** — `trufflehog filesystem .` should be clean
- [ ] **GitHub repo is set to public** (you're going public from day one)
- [ ] **Branch protection on `main`** — require PR reviews from collaborators, even though only you have access (it protects against the agent accidentally force-pushing)
