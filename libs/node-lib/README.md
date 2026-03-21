# node-lib

Minimal TypeScript library scaffold with self-contained verification and build scripts.

## Contracts

- `npm run lint` validates the public API source shape.
- `npm run typecheck` validates the exported TypeScript contract.
- `npm run build` emits ESM and declaration files into `dist/`.
- `npm run test` imports the built artifact and verifies the public API.
- `npm run package` validates the npm tarball shape.
