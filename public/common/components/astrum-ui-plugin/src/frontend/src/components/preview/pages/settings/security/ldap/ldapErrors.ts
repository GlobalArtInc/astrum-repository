import LdapStrings from '../../../../constants/pages/settings/security/LdapStrings';

export interface ReadableLdapError {
  /** What went wrong, in the page language */
  summary: string;
  /** The reason the server gives, without the LDAP SDK's version stamp; absent when it adds nothing */
  detail?: string;
}

/**
 * Turns the message of a failed LDAP check into something people can read. The server passes on the LDAP SDK's text,
 * e.g. "Could not connect to ldap://host:389: An error occurred while attempting to resolve address 'host':
 * UnknownHostException(host: Name or service not known), ldapSDKVersion=6.0.9, revision=42839ddf...": the version
 * stamp and the Java exception names mean nothing to an administrator.
 */
export function readableLdapError(message: string | null | undefined, fallback: string): ReadableLdapError {
  const cleaned = (message || '')
    .replace(/,?\s*ldapSDKVersion=[^,\s]*(,\s*revision=[0-9a-f]+)?/gi, '')
    .replace(/\b[A-Za-z]+Exception\((.*)\)/g, '$1')
    .trim();
  if (!cleaned) {
    return { summary: fallback };
  }
  const connect = /^Could not connect to (\S+?):\s+([\s\S]+)$/.exec(cleaned);
  if (connect) {
    return { summary: LdapStrings.FORM.COULD_NOT_CONNECT_TO(connect[1]), detail: connect[2] };
  }
  return { summary: fallback, detail: cleaned };
}
