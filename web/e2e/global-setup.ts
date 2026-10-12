import { startServer } from './server';

export default async function globalSetup() {
  const server = await startServer();
  process.env.ASTRUM_E2E_URL = server.url;
  process.env.ASTRUM_E2E_ADMIN_PASSWORD = server.adminPassword;
  return async () => {
    await server.stop();
  };
}
