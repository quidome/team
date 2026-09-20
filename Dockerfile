FROM node:22-bookworm-slim AS dependencies

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

FROM dependencies AS build

COPY . .
RUN npm run check
RUN npm run build

FROM build AS migration

CMD ["npm", "run", "db:migrate"]

FROM build AS runtime-dependencies

RUN npm prune --omit=dev

FROM node:22-bookworm-slim AS runtime

ENV HOST=0.0.0.0
ENV NODE_ENV=production
ENV PORT=3000

WORKDIR /app

COPY --from=runtime-dependencies --chown=node:node /app/build ./build
COPY --from=runtime-dependencies --chown=node:node /app/node_modules ./node_modules
COPY --from=runtime-dependencies --chown=node:node /app/package.json ./package.json

USER node
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD node -e "fetch('http://127.0.0.1:' + (process.env.PORT || 3000) + '/api/health/liveness').then((response) => { if (!response.ok) process.exit(1); }).catch(() => process.exit(1));"

CMD ["node", "build"]
