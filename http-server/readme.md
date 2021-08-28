# PCloud Http Server

Proxy server to have a static file server from your pcloud instance

## How to build and run

```bash
# to run in root folder
docker build -t pcloud-http-server -f http-server/Dockerfile .
# and run the container
docker run -d \
	-p 3000:3000 \
	-e RUST_LOG=info \
	-e PCLOUD_REGION=eu \
	-e PCLOUD_USERNAME=username \
	-e PCLOUD_PASSWORD=password \
	pcloud-http-server
# display content
curl http://localhost:3000/
```

