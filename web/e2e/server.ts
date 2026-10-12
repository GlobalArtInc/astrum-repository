import { type ChildProcess, execFileSync, spawn } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

export interface Running {
  url: string;
  adminPassword: string;
  stop(): Promise<void>;
}

const ROOT = resolve(import.meta.dirname, '..');

function freePort(): Promise<number> {
  return new Promise((done, fail) => {
    const server = createServer();
    server.once('error', fail);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      server.close(() => done(port));
    });
  });
}

function databaseUrl(base: string, name: string): string {
  const url = new URL(base);
  url.pathname = `/${name}`;
  return url.toString();
}

function psql(url: string, command: string): void {
  execFileSync('psql', [url, '-v', 'ON_ERROR_STOP=1', '-qc', command], { stdio: 'pipe' });
}

async function waitFor(url: string, child: ChildProcess, log: () => string): Promise<void> {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`the server stopped with ${child.exitCode}:\n${log()}`);
    }
    try {
      const response = await fetch(`${url}/service/rest/v1/status`);
      if (response.ok) {
        return;
      }
    } catch {}
    await new Promise((done) => setTimeout(done, 250));
  }
  throw new Error(`the server did not answer within a minute:\n${log()}`);
}

export interface StartOptions {
  firstRun?: boolean;
  port?: number;
}

export async function startServer(options: StartOptions = {}): Promise<Running> {
  const binary = process.env.ASTRUM_BIN ?? '/home/user/wt/m1-target/debug/astrum';
  const webDir = process.env.ASTRUM_WEB_DIR ?? join(ROOT, 'dist');
  if (!existsSync(binary)) {
    throw new Error(`no server binary at ${binary}; build it or set ASTRUM_BIN`);
  }
  if (!existsSync(join(webDir, 'index.html'))) {
    throw new Error(`no built UI in ${webDir}; run "pnpm build" first`);
  }
  const admin = process.env.ASTRUM_E2E_DATABASE_URL ?? 'postgres://postgres@127.0.0.1:5433/postgres';
  const prefix = process.env.ASTRUM_E2E_DATABASE_PREFIX ?? 'astrum_e2e';
  const database = `${prefix}_${randomBytes(4).toString('hex')}`;
  const dataDir = mkdtempSync(join(tmpdir(), 'astrum-e2e-'));
  const asked = options.port ?? Number(process.env.ASTRUM_E2E_PORT ?? 0);
  const port = asked > 0 ? asked : await freePort();
  const url = `http://127.0.0.1:${port}`;
  psql(admin, `create database ${database}`);
  let output = '';
  const child = spawn(binary, ['serve'], {
    env: {
      ...process.env,
      ASTRUM_DATA_DIR: dataDir,
      ASTRUM_LISTEN: `127.0.0.1:${port}`,
      ASTRUM_DATABASE_URL: databaseUrl(admin, database),
      ASTRUM_WEB_DIR: webDir,
      ASTRUM_LOG: process.env.ASTRUM_LOG ?? 'warn',
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  child.stdout?.on('data', (chunk: Buffer) => {
    output += chunk.toString();
  });
  child.stderr?.on('data', (chunk: Buffer) => {
    output += chunk.toString();
  });
  const cleanUp = () => {
    try {
      psql(admin, `drop database if exists ${database} with (force)`);
    } catch {}
    rmSync(dataDir, { recursive: true, force: true });
  };
  try {
    await waitFor(url, child, () => output);
  } catch (error) {
    child.kill('SIGTERM');
    cleanUp();
    throw error;
  }
  const passwordFile = join(dataDir, 'admin.password');
  const adminPassword = readFileSync(passwordFile, 'utf8').trim();
  if (!options.firstRun) {
    rmSync(passwordFile);
  }
  return {
    url,
    adminPassword,
    stop: async () => {
      if (child.exitCode === null) {
        const exited = new Promise((done) => child.once('exit', done));
        child.kill('SIGTERM');
        await Promise.race([exited, new Promise((done) => setTimeout(done, 10_000))]);
      }
      cleanUp();
    },
  };
}
