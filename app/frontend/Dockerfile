FROM node:21.5.0-alpine3.19
RUN npm i -g pnpm
WORKDIR /opt/app/frontend
COPY index.html .
COPY package.json .
COPY pnpm-lock.yaml .
RUN pnpm i
CMD ["pnpm", "start"]
