

export interface ServiceAccountToken {
  id: string;
  name: string;
  description: string;
  roleId: string;
  createdBy: string;
  createdAt: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
}

export interface CreateTokenForm {
  name: string;
  description: string;
  roleId: string;
  expirationDays?: number;
}

export interface RoleOption {
  id: string;
  name: string;
}
