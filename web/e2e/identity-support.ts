import { type ChildProcess, spawn } from 'node:child_process';
import { createHash, createSign, generateKeyPairSync, type KeyObject, randomBytes } from 'node:crypto';
import { existsSync } from 'node:fs';
import { createServer, type IncomingMessage, type Server, type ServerResponse } from 'node:http';
import type { AddressInfo } from 'node:net';

export const LDAP_BASE = 'dc=example,dc=com';
export const LDAP_SYSTEM_DN = 'cn=svc,dc=example,dc=com';
export const LDAP_SYSTEM_PASSWORD = 'svc-secret';
export const LDAP_USERS = {
  alice: { password: 'alice-secret', name: 'Alice Smith', groups: ['developers', 'admins'] },
  bob: { password: 'bob-secret', name: 'Bob Jones', groups: ['developers'] },
} as const;

export interface LdapFixture {
  port: number;
  ldapsPort: number;
  stop(): Promise<void>;
}

export async function startLdap(): Promise<LdapFixture> {
  const binary = process.env.ASTRUM_TEST_LDAP_BIN ?? '/home/user/wt/m1-target/debug/astrum-test-ldap';
  if (!existsSync(binary)) {
    throw new Error(`no LDAP fixture at ${binary}; build the workspace or set ASTRUM_TEST_LDAP_BIN`);
  }
  const port = process.env.ASTRUM_E2E_LDAP_PORT ?? '0';
  const child: ChildProcess = spawn(binary, [port, ''], { stdio: ['ignore', 'pipe', 'pipe'] });
  const ports = await new Promise<{ port: number; ldapsPort: number }>((resolve, reject) => {
    let output = '';
    const timer = setTimeout(() => reject(new Error(`the LDAP fixture did not start: ${output}`)), 15_000);
    child.stdout?.on('data', (chunk: Buffer) => {
      output += chunk.toString();
      const line = output.split('\n').find((candidate) => candidate.startsWith('{'));
      if (line) {
        clearTimeout(timer);
        resolve(JSON.parse(line) as { port: number; ldapsPort: number });
      }
    });
    child.once('exit', (code) => {
      clearTimeout(timer);
      reject(new Error(`the LDAP fixture exited with ${code}: ${output}`));
    });
  });
  return {
    ...ports,
    stop: async () => {
      if (child.exitCode === null) {
        const exited = new Promise((done) => child.once('exit', done));
        child.kill('SIGINT');
        await Promise.race([exited, new Promise((done) => setTimeout(done, 5_000))]);
      }
    },
  };
}

export interface OidcClaims {
  sub: string;
  preferred_username: string;
  given_name?: string;
  family_name?: string;
  email?: string;
  groups?: string[];
}

interface Pending {
  claims: OidcClaims;
  nonce: string;
  challenge: string;
  redirectUri: string;
}

export interface OidcProvider {
  issuer: string;
  clientId: string;
  clientSecret: string;
  signInAs(claims: OidcClaims): void;
  tokenRequests(): number;
  stop(): Promise<void>;
}

function base64url(data: Buffer | string): string {
  return Buffer.from(data).toString('base64url');
}

function sign(key: KeyObject, kid: string, payload: object): string {
  const header = base64url(JSON.stringify({ alg: 'RS256', typ: 'JWT', kid }));
  const body = base64url(JSON.stringify(payload));
  const signer = createSign('RSA-SHA256');
  signer.update(`${header}.${body}`);
  return `${header}.${body}.${signer.sign(key).toString('base64url')}`;
}

async function readBody(request: IncomingMessage): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of request) {
    chunks.push(chunk as Buffer);
  }
  return Buffer.concat(chunks).toString('utf8');
}

function json(response: ServerResponse, status: number, body: unknown): void {
  response.writeHead(status, { 'content-type': 'application/json', 'cache-control': 'no-store' });
  response.end(JSON.stringify(body));
}

export async function startOidcProvider(
  clientId = 'astrum-e2e',
  clientSecret = 'e2e-client-secret',
): Promise<OidcProvider> {
  const { privateKey, publicKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });
  const kid = randomBytes(6).toString('hex');
  const jwk = { ...publicKey.export({ format: 'jwk' }), kid, use: 'sig', alg: 'RS256' };
  const codes = new Map<string, Pending>();
  const tokens = new Map<string, OidcClaims>();
  let user: OidcClaims = { sub: 'nobody', preferred_username: 'nobody' };
  let requests = 0;
  let issuer = '';

  const server: Server = createServer(async (request, response) => {
    const url = new URL(request.url ?? '/', issuer);
    if (url.pathname === '/.well-known/openid-configuration') {
      json(response, 200, {
        issuer,
        authorization_endpoint: `${issuer}/authorize`,
        token_endpoint: `${issuer}/token`,
        userinfo_endpoint: `${issuer}/userinfo`,
        jwks_uri: `${issuer}/jwks`,
        end_session_endpoint: `${issuer}/logout`,
        response_types_supported: ['code'],
        subject_types_supported: ['public'],
        id_token_signing_alg_values_supported: ['RS256'],
        code_challenge_methods_supported: ['S256'],
      });
      return;
    }
    if (url.pathname === '/jwks') {
      json(response, 200, { keys: [jwk] });
      return;
    }
    if (url.pathname === '/authorize') {
      const redirectUri = url.searchParams.get('redirect_uri') ?? '';
      const state = url.searchParams.get('state') ?? '';
      if (
        url.searchParams.get('client_id') !== clientId ||
        url.searchParams.get('code_challenge_method') !== 'S256'
      ) {
        response.writeHead(400).end('bad authorization request');
        return;
      }
      const code = randomBytes(16).toString('hex');
      codes.set(code, {
        claims: user,
        nonce: url.searchParams.get('nonce') ?? '',
        challenge: url.searchParams.get('code_challenge') ?? '',
        redirectUri,
      });
      const target = new URL(redirectUri);
      target.searchParams.set('code', code);
      target.searchParams.set('state', state);
      response.writeHead(302, { location: target.toString() }).end();
      return;
    }
    if (url.pathname === '/token' && request.method === 'POST') {
      requests += 1;
      const form = new URLSearchParams(await readBody(request));
      const authorization = request.headers.authorization ?? '';
      const [id, secret] = authorization.startsWith('Basic ')
        ? Buffer.from(authorization.slice(6), 'base64').toString('utf8').split(':').map(decodeURIComponent)
        : [form.get('client_id'), form.get('client_secret')];
      const pending = codes.get(form.get('code') ?? '');
      codes.delete(form.get('code') ?? '');
      const verifier = form.get('code_verifier') ?? '';
      const challenge = createHash('sha256').update(verifier).digest('base64url');
      if (id !== clientId || secret !== clientSecret) {
        json(response, 401, { error: 'invalid_client' });
        return;
      }
      if (!pending || pending.challenge !== challenge || pending.redirectUri !== form.get('redirect_uri')) {
        json(response, 400, { error: 'invalid_grant' });
        return;
      }
      const now = Math.floor(Date.now() / 1000);
      const accessToken = randomBytes(16).toString('hex');
      tokens.set(accessToken, pending.claims);
      json(response, 200, {
        access_token: accessToken,
        token_type: 'Bearer',
        expires_in: 300,
        id_token: sign(privateKey, kid, {
          ...pending.claims,
          iss: issuer,
          aud: clientId,
          iat: now,
          exp: now + 300,
          nonce: pending.nonce,
        }),
      });
      return;
    }
    if (url.pathname === '/userinfo') {
      const claims = tokens.get((request.headers.authorization ?? '').replace(/^Bearer /, ''));
      if (claims) {
        json(response, 200, claims);
      } else {
        json(response, 401, { error: 'invalid_token' });
      }
      return;
    }
    response.writeHead(404).end();
  });
  const port = Number(process.env.ASTRUM_E2E_OIDC_PORT ?? 0);
  await new Promise<void>((resolve) => server.listen(port, '127.0.0.1', resolve));
  issuer = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  return {
    issuer,
    clientId,
    clientSecret,
    signInAs: (claims) => {
      user = claims;
    },
    tokenRequests: () => requests,
    stop: () => new Promise((resolve) => server.close(() => resolve())),
  };
}
