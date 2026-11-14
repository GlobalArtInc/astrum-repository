export type PasskeyFailure = 'cancelled' | 'unsupported' | 'other';

export interface PasskeySuccess<T> {
  ok: true;
  value: T;
}

export interface PasskeyFailed {
  ok: false;
  failure: PasskeyFailure;
}

export type PasskeyResult<T> = PasskeySuccess<T> | PasskeyFailed;

export interface PasskeyDescriptor {
  id: string;
  type: string;
  transports?: string[];
}

export interface PasskeyUser {
  id: string;
  name: string;
  displayName: string;
}

export interface RegistrationOptions {
  challenge: string;
  user: PasskeyUser;
  excludeCredentials?: PasskeyDescriptor[];
  [field: string]: unknown;
}

export interface AssertionOptions {
  challenge: string;
  allowCredentials?: PasskeyDescriptor[];
  [field: string]: unknown;
}

export interface RegistrationResponse {
  attestationObject: string;
  clientDataJSON: string;
}

export interface AssertionResponse {
  authenticatorData: string;
  clientDataJSON: string;
  signature: string;
  userHandle: string | null;
}

export interface RegistrationCredential {
  id: string;
  rawId: string;
  type: string;
  response: RegistrationResponse;
  clientExtensionResults: AuthenticationExtensionsClientOutputs;
}

export interface AssertionCredential {
  id: string;
  rawId: string;
  type: string;
  response: AssertionResponse;
  clientExtensionResults: AuthenticationExtensionsClientOutputs;
}

export function isPasskeySupported(): boolean {
  return typeof window !== 'undefined' && 'PublicKeyCredential' in window;
}

export function fromBase64Url(value: string): ArrayBuffer {
  const base64 = value
    .replace(/-/g, '+')
    .replace(/_/g, '/')
    .padEnd(Math.ceil(value.length / 4) * 4, '=');
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes.buffer;
}

export function toBase64Url(buffer: ArrayBuffer): string {
  let binary = '';
  for (const byte of new Uint8Array(buffer)) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function descriptorFromJson(descriptor: PasskeyDescriptor): PublicKeyCredentialDescriptor {
  return {
    ...descriptor,
    type: 'public-key',
    id: fromBase64Url(descriptor.id),
  } as PublicKeyCredentialDescriptor;
}

export function registrationOptionsFromJson(
  options: RegistrationOptions,
): PublicKeyCredentialCreationOptions {
  return {
    ...options,
    challenge: fromBase64Url(options.challenge),
    user: { ...options.user, id: fromBase64Url(options.user.id) },
    excludeCredentials: options.excludeCredentials?.map(descriptorFromJson),
  } as PublicKeyCredentialCreationOptions;
}

export function assertionOptionsFromJson(options: AssertionOptions): PublicKeyCredentialRequestOptions {
  return {
    ...options,
    challenge: fromBase64Url(options.challenge),
    allowCredentials: options.allowCredentials?.map(descriptorFromJson),
  } as PublicKeyCredentialRequestOptions;
}

function registrationJson(credential: PublicKeyCredential): RegistrationCredential {
  const response = credential.response as AuthenticatorAttestationResponse;
  return {
    id: credential.id,
    rawId: toBase64Url(credential.rawId),
    type: credential.type,
    response: {
      attestationObject: toBase64Url(response.attestationObject),
      clientDataJSON: toBase64Url(response.clientDataJSON),
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
}

function assertionJson(credential: PublicKeyCredential): AssertionCredential {
  const response = credential.response as AuthenticatorAssertionResponse;
  return {
    id: credential.id,
    rawId: toBase64Url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: toBase64Url(response.authenticatorData),
      clientDataJSON: toBase64Url(response.clientDataJSON),
      signature: toBase64Url(response.signature),
      userHandle: response.userHandle ? toBase64Url(response.userHandle) : null,
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
}

function failureOf(error: unknown): PasskeyFailure {
  return error instanceof DOMException && error.name === 'NotAllowedError' ? 'cancelled' : 'other';
}

export async function createPasskey(
  options: RegistrationOptions,
): Promise<PasskeyResult<RegistrationCredential>> {
  if (!isPasskeySupported()) {
    return { ok: false, failure: 'unsupported' };
  }
  try {
    const credential = await navigator.credentials.create({
      publicKey: registrationOptionsFromJson(options),
    });
    if (!credential) {
      return { ok: false, failure: 'other' };
    }
    return { ok: true, value: registrationJson(credential as PublicKeyCredential) };
  } catch (error) {
    return { ok: false, failure: failureOf(error) };
  }
}

export async function getPasskey(options: AssertionOptions): Promise<PasskeyResult<AssertionCredential>> {
  if (!isPasskeySupported()) {
    return { ok: false, failure: 'unsupported' };
  }
  try {
    const credential = await navigator.credentials.get({
      publicKey: assertionOptionsFromJson(options),
    });
    if (!credential) {
      return { ok: false, failure: 'other' };
    }
    return { ok: true, value: assertionJson(credential as PublicKeyCredential) };
  } catch (error) {
    return { ok: false, failure: failureOf(error) };
  }
}
