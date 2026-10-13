import type { Translate } from '../../../i18n';
import type { Action } from '../api';

const TYPE_LABELS: Record<string, string> = {
  application: 'security.privilegeType.application',
  wildcard: 'security.privilegeType.wildcard',
  'repository-view': 'security.privilegeType.repositoryView',
  'repository-admin': 'security.privilegeType.repositoryAdmin',
  'repository-content-selector': 'security.privilegeType.repositoryContentSelector',
};

const TYPE_DESCRIPTIONS: Record<string, string> = {
  application: 'security.privilegeType.applicationText',
  wildcard: 'security.privilegeType.wildcardText',
  'repository-view': 'security.privilegeType.repositoryViewText',
  'repository-admin': 'security.privilegeType.repositoryAdminText',
  'repository-content-selector': 'security.privilegeType.repositoryContentSelectorText',
};

const ACTION_LABELS: Record<Action, string> = {
  READ: 'security.action.read',
  BROWSE: 'security.action.browse',
  EDIT: 'security.action.edit',
  ADD: 'security.action.add',
  DELETE: 'security.action.delete',
  RUN: 'security.action.run',
  ASSOCIATE: 'security.action.associate',
  DISASSOCIATE: 'security.action.disassociate',
  START: 'security.action.start',
  STOP: 'security.action.stop',
  ALL: 'security.action.all',
};

export function typeLabel(t: Translate, type: string): string {
  const key = TYPE_LABELS[type];
  return key ? t(key) : type;
}

export function typeDescription(t: Translate, type: string): string {
  const key = TYPE_DESCRIPTIONS[type];
  return key ? t(key) : '';
}

export function actionLabel(t: Translate, action: string): string {
  const key = ACTION_LABELS[action as Action];
  return key ? t(key) : action;
}
