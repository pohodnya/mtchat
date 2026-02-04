# Build SDK first
FROM node:20-alpine AS sdk-builder

WORKDIR /sdk
COPY mtchat-vue/package*.json ./
RUN npm install
COPY mtchat-vue/ ./
RUN npm run build

# Build example app
FROM node:20-alpine AS app-builder

WORKDIR /build

# Copy built SDK to expected location
COPY --from=sdk-builder /sdk ./mtchat-vue

# Copy example app
COPY mtchat-example/ ./mtchat-example

# Install dependencies (now file:../mtchat-vue will resolve correctly)
WORKDIR /build/mtchat-example
RUN npm install

# Build
RUN npm run build

# Production stage
FROM nginx:alpine

COPY --from=app-builder /build/mtchat-example/dist /usr/share/nginx/html
COPY mtchat-example/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
