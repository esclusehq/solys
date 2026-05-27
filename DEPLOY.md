# Deploy to EC2

> **IMPORTANT:** Sebelum deploy, selalu commit dan push perubahan ke GitHub dulu. Lihat [PUSH_COMMIT.md](./PUSH_COMMIT.md) untuk panduan lengkap.

## Remote Info

- **EC2 Tailscale IP:** 100.121.160.102
- **SSH Key:** `~/Downloads/sqrt1mReyhan.pem`
- **Remote Path:** `/home/ec2-user/escluse/`

## Deployment Methods

Ada 2 cara deploy:

| Method | Cara | Kapan Gunakan |
|--------|------|---------------|
| **AWS ECR** (Recommended) | Build → Push ke ECR → Server pull | Untuk production/automated |
| **rsync** (Legacy) | Build local → tar → rsync ke server | Untuk quick fix tanpa ECR |

---

## Deployment Flow (AWS ECR - Recommended)

```
1. Commit & Push ke GitHub (lihat PUSH_COMMIT.md)
2. Build Docker Image di local
3. Push ke AWS ECR
4. Server pull dari ECR & restart container
```

### AWS ECR Configuration (One-time)

```bash
# Install AWS CLI (jika belum)
brew install awscli  # macOS

# Configure IAM user credentials
aws configure --profile escluse-deploy

# Test connection
aws sts get-caller-identity --profile escluse-deploy
```

**ECR Registry:** `535237074507.dkr.ecr.ap-southeast-1.amazonaws.com`

---

## Deployment Flow (rsync - Legacy)

```
1. Commit & Push ke GitHub (lihat PUSH_COMMIT.md)
2. Build Docker Image
3. Save ke tar → rsync ke EC2
4. Load & deploy container
```

---

## Step 0: Commit & Push ke GitHub

**WAJIB dilakukan sebelum deploy!**

Lihat [PUSH_COMMIT.md](./PUSH_COMMIT.md) untuk panduan lengkap. Ringkasan:

### Checklist sebelum deploy:

- [ ] `landing-page-escluse/` diubah? → Commit & push dulu
- [ ] `docs/` diubah? → Commit & push dulu
- [ ] `api/` atau `worker/` diubah? → Commit & push dulu
- [ ] `app/` diubah? → Commit & push dulu
- [ ] `gateway/` atau `docker-compose.yml` diubah? → Commit & push dulu

### Quick Commit (untuk repo yang sudah ada git):

```bash
# Check status dulu
git status --short

# Stage semua perubahan
git add -A

# Commit dengan deskripsi
git commit -m "feat: [deskripsi perubahan]"

# Push ke GitHub
git push origin [branch-name]
```

---

## AWS ECR Deployment (Recommended)

### Prerequisites

1. AWS CLI configured dengan profile `escluse-deploy`
2. ECR repositories sudah dibuat:
   - `escluse-backend`
   - `escluse-landing`
   - `escluse-frontend`
   - `escluse-docs`

### Full Deployment Commands (Per Step)

#### Step 1: Login ke ECR (Local)

```bash
aws ecr get-login-password --region ap-southeast-1 --profile escluse-deploy | \
  docker login --username AWS --password-stdin \
  535237074507.dkr.ecr.ap-southeast-1.amazonaws.com
```

#### Step 2: Build Images

```bash
# Backend (Rust)
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/api
docker build -t escluse-backend:latest -f Dockerfile .

# Landing Page (Node/Vite) - menggunakan Dockerfile.landing di root
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse
docker build -f Dockerfile.landing -t escluse-landing:latest .

# Frontend Dashboard
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/app
set -a && source .env 2>/dev/null; set +a
docker build -t escluse-frontend:latest -f Dockerfile.prod \
  --build-arg VITE_SUPABASE_URL="$VITE_SUPABASE_URL" \
  --build-arg VITE_SUPABASE_ANON_KEY="$VITE_SUPABASE_ANON_KEY" \
  --build-arg VITE_API_URL="$VITE_API_URL" \
  .

# Docs (VitePress)
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/docs
docker build -t escluse-docs:latest .
```

**Catatan:** Docs harus punya Dockerfile di `docs/Dockerfile`:
```bash
cat > docs/Dockerfile << 'EOF'
FROM nginx:alpine
COPY .vitepress/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
EOF
```

#### Step 3: Tag Images untuk ECR

```bash
docker tag escluse-backend:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-backend:latest
docker tag escluse-landing:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-landing:latest
docker tag escluse-frontend:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-frontend:latest
docker tag escluse-docs:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-docs:latest
```

#### Step 4: Push ke ECR

```bash
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-backend:latest &
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-landing:latest &
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-frontend:latest &
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-docs:latest &
wait
```

#### Step 5: Copy docker-compose.yml ke EC2

```bash
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  /home/rhnbztnl/Downloads/Berguna/Projects/escluse/docker-compose.yml \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/
```

#### Step 6: Setup AWS Credentials di EC2 (One-time)

```bash
# Copy credentials dari local ke EC2
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  ~/.aws/ \
  ec2-user@100.121.160.102:/home/ec2-user/.aws/
```

#### Step 7: Pull & Deploy di EC2

```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no ec2-user@100.121.160.102 \
  "cd escluse && \
   aws ecr get-login-password --region ap-southeast-1 --profile escluse-deploy | docker login --username AWS --password-stdin 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com && \
   docker compose pull && \
   docker compose up -d"
```

### One-liner Full Deploy (Semua Sekaligus)

```bash
aws ecr get-login-password --region ap-southeast-1 --profile escluse-deploy | \
  docker login --username AWS --password-stdin 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com && \
\
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/api && \
docker build -t escluse-backend:latest -f Dockerfile . && \
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse && \
docker build -f Dockerfile.landing -t escluse-landing:latest . && \
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/app && \
set -a && source .env 2>/dev/null; set +a && \
docker build -t escluse-frontend:latest -f Dockerfile.prod \
  --build-arg VITE_SUPABASE_URL="$VITE_SUPABASE_URL" \
  --build-arg VITE_SUPABASE_ANON_KEY="$VITE_SUPABASE_ANON_KEY" \
  --build-arg VITE_API_URL="$VITE_API_URL" \
  . && \
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/docs && \
docker build -t escluse-docs:latest . && \
\
docker tag escluse-backend:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-backend:latest && \
docker tag escluse-landing:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-landing:latest && \
docker tag escluse-frontend:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-frontend:latest && \
docker tag escluse-docs:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-docs:latest && \
\
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-backend:latest & \
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-landing:latest & \
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-frontend:latest & \
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-docs:latest & \
wait && \
\
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  /home/rhnbztnl/Downloads/Berguna/Projects/escluse/docker-compose.yml \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/ && \
ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no ec2-user@100.121.160.102 \
  "cd escluse && aws ecr get-login-password --region ap-southeast-1 | docker login --username AWS --password-stdin 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com && docker compose pull && docker compose up -d"
```

### Quick Deploy (Landing Page Saja)

Untuk perubahan kecil di landing page saja (tanpa rebuild backend/frontend):

```bash
# Step 1: Build landing page (wajib rebuild dist dulu)
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse/landing-page-escluse
rm -rf dist && npm run build

# Step 2: Build & push Docker image
cd /home/rhnbztnl/Downloads/Berguna/Projects/escluse
docker build --no-cache -f Dockerfile.landing -t escluse-landing:latest .
docker tag escluse-landing:latest 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-landing:latest
docker push 535237074507.dkr.ecr.ap-southeast-1.amazonaws.com/escluse-landing:latest

# Step 3: Deploy ke EC2
ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no ec2-user@100.121.160.102 \
  "docker rm -f escluse_landing; cd escluse && docker compose pull landing && docker compose up -d landing"
```

### Create ECR Repository (One-time)

```bash
aws ecr create-repository --repository-name escluse-backend --region ap-southeast-1 --profile escluse-deploy
aws ecr create-repository --repository-name escluse-landing --region ap-southeast-1 --profile escluse-deploy
aws ecr create-repository --repository-name escluse-frontend --region ap-southeast-1 --profile escluse-deploy
aws ecr create-repository --repository-name escluse-docs --region ap-southeast-1 --profile escluse-deploy
```

**ECR Registry:** `535237074507.dkr.ecr.ap-southeast-1.amazonaws.com`

---

## rsync Deployment (Legacy)

### Prerequisite: Build Landing Page di Local

```bash
cd landing-page-escluse
npm install
npm run build
cd ..
```

### Step 1: Build Docker Image

```bash
docker build -f Dockerfile.landing -t escluse-landing:latest .
```

### Step 2: Save Image to tar

```bash
docker save escluse-landing:latest -o /tmp/escluse-landing.tar
```

### Step 3: Upload to EC2

```bash
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  /tmp/escluse-landing.tar \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/
```

### Step 4: Load and Deploy on EC2

```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "cd escluse && \
  docker load -i escluse-landing.tar && \
  docker compose up -d landing && \
  rm -f escluse-landing.tar"
```

### Step 5: Cleanup local tar

```bash
rm -f /tmp/escluse-landing.tar
```

### One-liner Landing Page Deploy (SETELAH commit & push)

```bash
# Commit & push dulu! (dari PUSH_COMMIT.md)
cd landing-page-escluse && git add -A && git commit -m "feat: [description]" && git push origin master && cd ..

# Build & deploy
docker build -f Dockerfile.landing -t escluse-landing:latest . && \
docker save escluse-landing:latest -o /tmp/escluse-landing.tar && \
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  /tmp/escluse-landing.tar \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/ && \
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 \
  "cd escluse && docker load -i escluse-landing.tar && docker compose up -d landing && rm -f escluse-landing.tar" && \
rm -f /tmp/escluse-landing.tar
```

---

## Docs Deployment

Docs menggunakan VitePress dan di-deploy via volume mount ke nginx.

### Step 0: Commit & Push Docs (WAJIB!)

```bash
cd docs
git add -A
git commit -m "docs: [description]"
git push origin master
cd ..
```

### Step 1: Build Docs

```bash
cd docs && npm run build && cd ..
```

### Step 2: Sync dan Restart Container

```bash
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  --delete \
  docs/.vitepress/dist/ \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/docs/

ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "cd escluse && docker compose restart docs"
```

### One-liner Docs Deploy (SETELAH commit & push)

```bash
# Commit & push dulu!
cd docs && git add -A && git commit -m "docs: [description]" && git push origin master && cd ..

# Build & deploy
npm run build && \
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" --delete \
  docs/.vitepress/dist/ \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/docs/ && \
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "cd escluse && docker compose restart docs"
```

---

## Backend & Frontend Deployment

### Step 0: Commit & Push (WAJIB!)

```bash
# Backend (api/ + worker/)
cd api
git add -A
git commit -m "feat: [description]"
git push origin master
cd ..

# Frontend (app/)
cd app
git add -A
git commit -m "feat: [description]"
git push origin master
cd ..
```

### Step 1: Build images locally

```bash
docker build -f api/Dockerfile -t escluse-backend:latest api/
docker build -f app/Dockerfile.prod -t escluse-frontend:latest app/
```

### Step 2: Save images to tar

```bash
docker save escluse-backend:latest -o /tmp/escluse-backend.tar
docker save escluse-frontend:latest -o /tmp/escluse-frontend.tar
```

### Step 3: Upload tar files to EC2 via rsync

```bash
rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  /tmp/escluse-backend.tar \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/

rsync -avz -e "ssh -i ~/Downloads/sqrt1mReyhan.pem -o StrictHostKeyChecking=no" \
  /tmp/escluse-frontend.tar \
  ec2-user@100.121.160.102:/home/ec2-user/escluse/
```

### Step 4: Load images on EC2

```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "cd escluse && \
  docker load -i escluse-backend.tar && \
  docker load -i escluse-frontend.tar"
```

### Step 5: Start containers on EC2

```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "cd escluse && docker compose up -d"
```

---

## Cleanup

```bash
rm -f /tmp/escluse-backend.tar /tmp/escluse-frontend.tar /tmp/escluse-landing.tar

ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "docker image prune -a"
```

---

## Troubleshooting

### Check containers status
```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "docker ps"
```

### Check landing page logs
```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "docker compose logs -f landing"
```

### Check backend logs
```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "docker compose logs -f backend"
```

### Rebuild dari scratch
```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102 "cd escluse && docker builder prune -a && docker compose up -d"
```

---

## Notes

- Pastikan Tailscale running di EC2 dan local
- EC2 Docker builder bermasalah untuk Rust builds, jadi kita build locally
- docker-compose.yml menggunakan images dari AWS ECR
- Healthchecks sudah dikonfigurasi untuk postgres dan redis
- Landing page accessible via port 8081 atau melalui Caddy di esluce.com
- Docs sekarang menggunakan Docker image dari ECR (bukan volume mount)
- Estimated build time: ~15-20 menit (terutama backend karena compile Rust)
- Push ke ECR bisa bersamaan dengan `&` dan `wait`

---

## GitHub Repositories

Lihat [PUSH_COMMIT.md](./PUSH_COMMIT.md) untuk mapping lengkap.

| Repo | URL |
|------|-----|
| Landing Page | https://github.com/esclusehq/escluse-landing-page |
| Documentation | https://github.com/esclusehq/escluse-docs |
| Dashboard | https://github.com/esclusehq/escluse-dashboard |
| Backend | https://github.com/esclusehq/escluse-cloud |
| Infrastructure | https://github.com/esclusehq/escluse-infra |

**GitHub Organization:** https://github.com/esclusehq

**Git Config (one-time setup per repo):**
```bash
git config user.email "dev@esluce.com"
git config user.name "Escluse Dev"
```