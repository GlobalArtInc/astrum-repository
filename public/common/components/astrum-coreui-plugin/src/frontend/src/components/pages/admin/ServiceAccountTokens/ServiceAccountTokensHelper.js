
import {APIConstants, ExtJS, Permissions} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {REST: {PUBLIC: {SERVICE_ACCOUNT_TOKENS: serviceAccountTokensUrl, ROLES: rolesUrl}}} = APIConstants;
const {SERVICE_ACCOUNT_TOKENS: {CREATE_MODAL: CREATE_LABELS, MESSAGES}} = UIStrings;

const singleTokenUrl = (id) => `${serviceAccountTokensUrl}/${encodeURIComponent(id)}`;

export const URL = {serviceAccountTokensUrl, singleTokenUrl, rolesUrl};

export const TOKEN_MODAL_AUTO_CLOSE_SECONDS = 60;

export const EXPIRATION_OPTIONS = [
  {value: 30, label: CREATE_LABELS.EXPIRATION_OPTIONS.THIRTY_DAYS},
  {value: 60, label: CREATE_LABELS.EXPIRATION_OPTIONS.SIXTY_DAYS},
  {value: 90, label: CREATE_LABELS.EXPIRATION_OPTIONS.NINETY_DAYS},
  {value: 365, label: CREATE_LABELS.EXPIRATION_OPTIONS.ONE_YEAR},
  {value: -1, label: CREATE_LABELS.EXPIRATION_OPTIONS.NEVER},
];

export const canCreateToken = () => ExtJS.checkPermission(Permissions.SERVICE_ACCOUNTS.CREATE);
export const canRevokeToken = () => ExtJS.checkPermission(Permissions.SERVICE_ACCOUNTS.DELETE);

export function mapCreateError(event) {
  const status = event.data?.response?.status;
  if (status === 403) return MESSAGES.CREATE_ERROR_FORBIDDEN;
  if (status === 400) {
    const body = event.data?.response?.data;
    const detail = typeof body === 'string' ? body : body?.message;
    return detail || MESSAGES.CREATE_ERROR_INVALID;
  }
  return MESSAGES.CREATE_ERROR_GENERIC;
}

export function mapRevokeError(event) {
  const status = event.data?.response?.status;
  if (status === 404) return MESSAGES.REVOKE_ERROR_NOT_FOUND;
  if (status === 403) return MESSAGES.REVOKE_ERROR_FORBIDDEN;
  return MESSAGES.REVOKE_ERROR_GENERIC;
}

export function mapRolesError(err) {
  if (err?.response?.status === 403) return CREATE_LABELS.ROLES_LOAD_ERROR_FORBIDDEN;
  return CREATE_LABELS.ROLES_LOAD_ERROR_GENERIC;
}
