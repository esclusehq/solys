# Phase 66 Discussion Log

**Date:** 2026-05-31

## Area: Deployment Architecture
- **Q:** How should Umami be deployed on EC2?
- **Options presented:** Docker on EC2 (Recommended), Source/build on EC2, Docker Compose stack
- **Answer:** Docker on EC2 (Recommended)

## Area: RDS Setup
- **Q:** New or existing RDS for Umami?
- **Options presented:** New RDS for Umami, Use existing RDS
- **Answer:** New RDS for Umami

## Area: Domain
- **Q:** What subdomain for the Umami dashboard?
- **Options presented:** analytics.esluce.com (Recommended), umami.esluce.com, stats.esluce.com
- **Answer:** analytics.esluce.com (Recommended)

## Area: Tracking Scope
- **Q:** Which Esluce properties should Umami track?
- **Options presented:** Landing page + App, Landing page only, All subdomains
- **Answer:** All subdomains

## Area: SSL & Proxy
- **Q:** Reverse proxy/SSL approach?
- **Options presented:** Caddy (Recommended), Nginx + Certbot
- **Answer:** Caddy (Recommended)

## Area: Automation
- **Q:** Deployment automation approach?
- **Options presented:** Manual + Docs (Recommended), Scripted setup, Terraform + Docker
- **Answer:** Manual + Docs (Recommended)
