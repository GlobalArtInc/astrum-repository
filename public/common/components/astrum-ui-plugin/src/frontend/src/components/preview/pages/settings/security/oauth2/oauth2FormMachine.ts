

import { assign } from 'xstate';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';
import ValidationUtils from '../../../../../../interface/ValidationUtils';
import UIStrings from '../../../../../../constants/UIStrings';
import { OAuth2Config, DEFAULT_OAUTH2_CONFIG } from './types';
import { fetchOAuth2Config, saveOAuth2Config } from './oauth2Api';

const REQUIRED_FIELDS = ['clientId'] as const;

const OPTIONAL_URL_FIELDS = ['idpAuthorizationUrl', 'idpLogoutUrl', 'idpTokenUrl', 'idpJwksUrl'] as const;

/**
 * Optional fields sent to the API as JSON objects. Blank is allowed (becomes
 * `{}`), but non-blank input must parse to a JSON object.
 */
const JSON_OBJECT_FIELDS = ['exactMatchClaims', 'authorizationCustomParams', 'tokenRequestCustomParams'] as const;

/**
 * True when `value` parses as a flat JSON object whose values are all strings.
 *
 * The API models these fields as `Map<String, String>`
 * (`OAuth2OidcConfigurationXO`), so arrays and nested objects are rejected — the
 * backend answers 400 for those, which surfaces only as a generic save error.
 * Non-string scalars are rejected too: Jackson silently coerces them (`5` is
 * stored as `"5"`), so rejecting keeps what is saved equal to what was typed.
 */
function isStringMap(value: string): boolean {
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    return false;
  }

  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    return false;
  }
  return Object.values(parsed).every((entry) => typeof entry === 'string');
}

/**
 * Validate OAuth2 form data.
 *
 * Delegates to the shared `ValidationUtils` so both the rules and the message
 * text stay identical to the Classic page (ASTRUM-54266) — previously this
 * reimplemented blank/URL checks with a local regex and bespoke per-field
 * strings, which drifted from Classic's shared `UIStrings.ERROR` messages.
 */
export function validateOAuth2(data: OAuth2Config): ValidationErrors {
  const errors: ValidationErrors = {};

  for (const name of REQUIRED_FIELDS) {
    const error = ValidationUtils.validateNotBlank(data[name]);
    if (error) errors[name] = error;
  }

  for (const name of OPTIONAL_URL_FIELDS) {
    const error = data[name]?.trim() ? ValidationUtils.validateIsUrl(data[name]) : null;
    if (error) errors[name] = error;
  }

  const hasManualEndpoints = Boolean(data.idpAuthorizationUrl?.trim() && data.idpTokenUrl?.trim()
      && (data.idpJwksUrl?.trim() || data.idpJwks?.trim()));
  if (data.issuerUrl?.trim()) {
    const error = ValidationUtils.validateIsUrl(data.issuerUrl);
    if (error) errors.issuerUrl = error;
  } else if (!hasManualEndpoints) {
    errors.issuerUrl = UIStrings.ERROR.FIELD_REQUIRED;
  }

  // ASTRUM-54266: reject malformed JSON rather than letting the API layer coerce
  // it to `{}`, which reported a successful save while destroying the previous
  // value. Classic aborts the save by throwing from JSON.parse; validating here
  // rejects earlier and points at the offending field.
  for (const name of JSON_OBJECT_FIELDS) {
    const value = data[name];
    if (value?.trim() && !isStringMap(value)) {
      errors[name] = UIStrings.ERROR.INVALID_JSON_OBJECT;
    }
  }

  return errors;
}

/**
 * OAuth2 settings form machine.
 *
 * Standard settings form: loads the current config, validates on the client,
 * and saves. `stayEditableAfterSave` keeps the form editable after a successful
 * save (no post-save navigation), matching the legacy page.
 *
 * Load/save are `invoke`d services (automatic cancellation on state exit).
 */
export function createOAuth2FormMachine() {
  return createFormMachine<OAuth2Config>({
    id: 'oauth2-form',
    context: {
      data: { ...DEFAULT_OAUTH2_CONFIG },
    } as FormContext<OAuth2Config>,
    stayEditableAfterSave: true,
    actions: {
      validate: assign((ctx: FormContext<OAuth2Config>) => ({
        validationErrors: validateOAuth2(ctx.data),
      })),
    },
    services: {
      load: async () => ({ data: await fetchOAuth2Config() }),
      save: async (ctx) => saveOAuth2Config(ctx.data),
    },
  });
}
