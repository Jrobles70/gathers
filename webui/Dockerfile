FROM node:22-alpine as builder

ARG REACT_APP_COLLECTIONS_ENABLED=false
ENV REACT_APP_COLLECTIONS_ENABLED=$REACT_APP_COLLECTIONS_ENABLED

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm install
RUN npm ci

COPY public ./public
COPY src ./src

RUN npm run build

FROM nginx:alpine

COPY --from=builder /app/build /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 3000

CMD ["nginx", "-g", "daemon off;"]
