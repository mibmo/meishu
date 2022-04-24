# Meishu
Leaderboard with basic verification of pending scores.

## Building
Builds are compatible with the Rust stable and target a MSRV of 1.59.0, but may be compatible with earlier versions.
```sh
cargo build --release
```

Container are automatically built every release and are available at the [Github Container Registry](https://ghcr.io/mibmo/meishu), but can also be built manually using a compatible tool (buildah, podman, docker).
```sh
buildah bud -f ./Containerfile -t meishu
podman build . -t meishu
docker build . -t meishu
```

## Deploying
A docker-compatible `compose.yaml` file is provided for easy deployment, but really all you need is a running Postgres database and setting up a few environment variables before running the leaderboard.
```sh
DB_HOST=localhost \
DB_PORT=5432 \
DB_NAME=leaderboard \
DB_USER=meishu \
DB_PASS=meishu \
meishu
```

## Testing
To create a few "test" scores, send `POST` requests to the score creation endpoint `/api/score` with the following JSON body, upon which the score's ID is returned.
```json
{
    "username": "hello",
    "score": 420
}
```
The `username` field may be omitted, and this results in a pending score being submitted.
The pending score can then be subsequently verified by visiting `/score/{id}`, or `/pending` for the latest pending score.

## API documentation
The API is documented in `openapi.yaml`, and any openapi compatible tool can be used to generate interactive testing suites (i.e. Insomnia, Postman, Swagger).

Although the API is completely documented, *regular* endpoints are not.
- Leaderboard: `/`
- Verify or view specific score: `/score/{id}`
- Verify latest pending score: `/pending`
