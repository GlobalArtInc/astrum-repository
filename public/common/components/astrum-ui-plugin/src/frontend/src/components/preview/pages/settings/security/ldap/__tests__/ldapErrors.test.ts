import { readableLdapError } from '../ldapErrors';

describe('readableLdapError', () => {
  it('splits a failed connection into where and why, without the SDK version stamp and exception names', () => {
    const result = readableLdapError(
      "Could not connect to ldap://ldap.invalid:389: An error occurred while attempting to resolve address 'ldap.invalid': " +
        'UnknownHostException(ldap.invalid: Name or service not known), ldapSDKVersion=6.0.9, revision=42839ddf0d77d954805fbbe3cce73a792af40474',
      'Connection verification failed'
    );

    expect(result.summary).toBe('Could not connect to ldap://ldap.invalid:389');
    expect(result.detail).toBe(
      "An error occurred while attempting to resolve address 'ldap.invalid': ldap.invalid: Name or service not known"
    );
  });

  it('keeps other messages as the detail under the fallback summary', () => {
    expect(readableLdapError('Invalid credentials', 'User mapping verification failed')).toEqual({
      summary: 'User mapping verification failed',
      detail: 'Invalid credentials',
    });
  });

  it('falls back when there is no message', () => {
    expect(readableLdapError(undefined, 'Connection verification failed')).toEqual({
      summary: 'Connection verification failed',
    });
  });
});
