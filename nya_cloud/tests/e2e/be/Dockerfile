FROM --platform=linux/amd64 oven/bun:slim
WORKDIR /src
COPY . .
RUN bun install --frozen-lockfile
CMD [ "bun", "run", "index.js" ]