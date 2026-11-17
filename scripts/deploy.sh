#!/usr/bin/env bash
# Deploy Loom on thinkcentre. Run from the repo root on that host.
set -euo pipefail
cd "$(dirname "$0")/.."
sha=$(git rev-parse --short HEAD)
app_port=8079
web_port=8082
env_file="$HOME/.config/loom/env"
backup_dir="$HOME/backups/loom"
mkdir -p "$backup_dir" "$(dirname "$env_file")"

if sudo -u postgres psql -tAc "select 1 from pg_database where datname='loom'" | grep -q 1; then
  name="loom-predeploy-$(date -u +%Y%m%dT%H%M%SZ)-$sha"
  sudo -u postgres pg_dump -Fc loom > "$backup_dir/$name.dump"
  sudo -u postgres pg_restore -l "$backup_dir/$name.dump" >/dev/null
  echo "backup: PASS ($backup_dir/$name.dump)"
else
  echo "backup: bootstrap (no prior loom database)"
  if [ ! -f "$env_file" ]; then
    password=$(openssl rand -hex 24)
    printf 'DATABASE_URL=postgres://loom:%s@127.0.0.1:5432/loom\n' "$password" > "$env_file"
    chmod 600 "$env_file"
    sudo -u postgres psql -c "create role loom login password '$password'" >/dev/null
    sudo -u postgres createdb -O loom loom
    echo "database: created loom owned by loom"
  fi
fi

echo "build: server"
CARGO_TARGET_DIR="$HOME/.cargo-target/loom" cargo build --release -p loom-api --bin loom-server
echo "build: frontend"
bash scripts/sync-chrome.sh
(cd frontend && npm ci --no-audit --no-fund >/dev/null && node node_modules/vite/bin/vite.js build >/dev/null)

mkdir -p "$HOME/.local/share/loom"
install -m 755 "$HOME/.cargo-target/loom/release/loom-server" "$HOME/.local/bin/loom-server"
rm -rf "$HOME/.local/share/loom/build"
cp -r frontend/build "$HOME/.local/share/loom/build"

mkdir -p "$HOME/.config/systemd/user"
cat > "$HOME/.config/systemd/user/loom.service" <<UNIT
[Unit]
Description=Loom feed directory API
After=network.target

[Service]
Type=simple
EnvironmentFile=$env_file
ExecStart=$HOME/.local/bin/loom-server
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
UNIT
systemctl --user daemon-reload
systemctl --user enable --now loom.service
healthy=0
for i in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:$app_port/api/v1/capabilities" >/dev/null 2>&1; then healthy=1; break; fi
  sleep 1
done
if [ "$healthy" -ne 1 ]; then
  echo "deploy: API never became healthy on $app_port" >&2
  systemctl --user status loom.service --no-pager -l >&2 || true
  exit 1
fi
echo "health: PASS on $app_port"

sudo tee /etc/nginx/sites-available/loom >/dev/null <<NGINX
server {
    listen $web_port;
    server_name loom.polarisocial.xyz;
    include /etc/nginx/snippets/estate-icons.conf;
    include /etc/nginx/snippets/estate-chrome.conf;
    location /api/ {
        proxy_pass http://127.0.0.1:$app_port;
        proxy_http_version 1.1;
        proxy_set_header Host \\\$host;
        proxy_set_header X-Real-IP \\\$remote_addr;
        proxy_set_header Accept-Encoding "";
    }
    location /b/ {
        proxy_pass http://127.0.0.1:$app_port;
        proxy_set_header Accept-Encoding "";
    }
    location / {
        root $HOME/.local/share/loom/build;
        try_files \\\$uri \\\$uri.html /200.html;
    }
    location = /200.html {
        root $HOME/.local/share/loom/build;
        internal;
    }
}
NGINX
sudo ln -sf /etc/nginx/sites-available/loom /etc/nginx/sites-enabled/loom
sudo nginx -t
sudo systemctl reload nginx

echo "verify: live paths"
curl -fsS "http://127.0.0.1:$app_port/api/v1/capabilities" | python3 -m json.tool | head -8
html=$(curl -fsS "http://127.0.0.1:$web_port/")
printf '%s' "$html" | grep -q 'data-loom-ssr="1"'
printf '%s' "$html" | grep -q 'Worth a look'
curl -fsS "http://127.0.0.1:$web_port/api/v1/capabilities" >/dev/null
curl -fsS "http://127.0.0.1:$web_port/manifest.webmanifest" >/dev/null
echo "live: PASS (HTML + API + PWA manifest on $web_port; app on $app_port)"
echo "deploy: $sha"
