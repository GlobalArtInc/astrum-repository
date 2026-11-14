import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  assertionOptionsFromJson,
  createPasskey,
  fromBase64Url,
  getPasskey,
  isPasskeySupported,
  registrationOptionsFromJson,
  toBase64Url,
} from './webauthn';

function bytes(...values: number[]): ArrayBuffer {
  return new Uint8Array(values).buffer;
}

function view(buffer: ArrayBuffer | undefined): number[] {
  return Array.from(new Uint8Array(buffer ?? new ArrayBuffer(0)));
}

function fakeCredentials(create: () => unknown, get: () => unknown) {
  const credentials = { create: vi.fn(async () => create()), get: vi.fn(async () => get()) };
  Object.defineProperty(navigator, 'credentials', { configurable: true, value: credentials });
  return credentials;
}

function fakeRegistration() {
  return {
    id: 'AQI',
    rawId: bytes(1, 2),
    type: 'public-key',
    response: { attestationObject: bytes(3, 4), clientDataJSON: bytes(5, 6) },
    getClientExtensionResults: () => ({}),
  };
}

function fakeAssertion(userHandle: ArrayBuffer | null) {
  return {
    id: 'AQI',
    rawId: bytes(1, 2),
    type: 'public-key',
    response: {
      authenticatorData: bytes(7),
      clientDataJSON: bytes(8),
      signature: bytes(9, 10),
      userHandle,
    },
    getClientExtensionResults: () => ({}),
  };
}

function failure(name: string): DOMException {
  return new DOMException('refused', name);
}

afterEach(() => {
  vi.unstubAllGlobals();
  Reflect.deleteProperty(navigator, 'credentials');
});

describe('base64url', () => {
  it.each([
    ['one byte, two padding signs', [0x41]],
    ['two bytes, one padding sign', [0x41, 0x42]],
    ['three bytes, no padding', [0x41, 0x42, 0x43]],
    ['characters that standard base64 writes as plus and slash', [0xfb, 0xff, 0xbf]],
    ['no bytes', []],
  ])('round-trips %s', (_name, values) => {
    const encoded = toBase64Url(bytes(...values));
    expect(encoded).not.toMatch(/[+/=]/);
    expect(view(fromBase64Url(encoded))).toEqual(values);
  });
});

describe('the options conversion', () => {
  it('converts the challenge, the user id and every excluded credential id', () => {
    const converted = registrationOptionsFromJson({
      challenge: 'AQI',
      user: { id: 'BAUG', name: 'admin', displayName: 'Administrator' },
      excludeCredentials: [{ id: 'Bgc', type: 'public-key', transports: ['internal'] }],
      rp: { id: 'localhost', name: 'Astrum' },
    });
    expect(view(converted.challenge as ArrayBuffer)).toEqual([1, 2]);
    expect(view(converted.user.id as ArrayBuffer)).toEqual([4, 5, 6]);
    expect(view(converted.excludeCredentials?.[0]?.id as ArrayBuffer)).toEqual([6, 7]);
    expect(converted.user.name).toBe('admin');
    expect(converted.rp).toEqual({ id: 'localhost', name: 'Astrum' });
  });

  it('converts the challenge and every allowed credential id of an assertion', () => {
    const converted = assertionOptionsFromJson({
      challenge: 'CAk',
      allowCredentials: [{ id: 'Cgs', type: 'public-key' }],
      userVerification: 'required',
    });
    expect(view(converted.challenge as ArrayBuffer)).toEqual([8, 9]);
    expect(view(converted.allowCredentials?.[0]?.id as ArrayBuffer)).toEqual([10, 11]);
    expect(converted.userVerification).toBe('required');
  });
});

describe('the ceremonies', () => {
  it('serialises a registration as the server expects it', async () => {
    vi.stubGlobal('PublicKeyCredential', class {});
    const credentials = fakeCredentials(fakeRegistration, () => null);
    const result = await createPasskey({
      challenge: 'AQI',
      user: { id: 'AQI', name: 'a', displayName: 'a' },
    });
    expect(result).toEqual({
      ok: true,
      value: {
        id: 'AQI',
        rawId: 'AQI',
        type: 'public-key',
        response: { attestationObject: 'AwQ', clientDataJSON: 'BQY' },
        clientExtensionResults: {},
      },
    });
    expect(credentials.create).toHaveBeenCalledOnce();
  });

  it('serialises an assertion, with a null user handle when the authenticator sends none', async () => {
    vi.stubGlobal('PublicKeyCredential', class {});
    fakeCredentials(
      () => null,
      () => fakeAssertion(null),
    );
    const result = await getPasskey({ challenge: 'AQI' });
    expect(result).toEqual({
      ok: true,
      value: {
        id: 'AQI',
        rawId: 'AQI',
        type: 'public-key',
        response: { authenticatorData: 'Bw', clientDataJSON: 'CA', signature: 'CQo', userHandle: null },
        clientExtensionResults: {},
      },
    });
  });

  it('reports a refused prompt as cancelled', async () => {
    vi.stubGlobal('PublicKeyCredential', class {});
    fakeCredentials(
      () => Promise.reject(failure('NotAllowedError')),
      () => null,
    );
    expect(
      await createPasskey({ challenge: 'AQI', user: { id: 'AQI', name: 'a', displayName: 'a' } }),
    ).toEqual({
      ok: false,
      failure: 'cancelled',
    });
  });

  it('reports a missing PublicKeyCredential as unsupported', async () => {
    fakeCredentials(fakeRegistration, () => null);
    expect(isPasskeySupported()).toBe(false);
    expect(await getPasskey({ challenge: 'AQI' })).toEqual({ ok: false, failure: 'unsupported' });
  });
});
