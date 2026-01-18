.PHONY: deploy-backend deploy-frontend deploy-all

BACKEND_DIR := app/backend
FRONTEND_DIR := app/frontend/yew-app

deploy-backend:
	@echo "Deploying backend to Fly.io..."
	cd $(BACKEND_DIR) && fly deploy

deploy-frontend:
	@echo "Deploying frontend to Cloudflare Pages..."
	cd $(FRONTEND_DIR) && npm run build:deploy

deploy-all: deploy-backend deploy-frontend
	@echo "Backend and frontend deployment finished."
