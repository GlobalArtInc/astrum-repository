

export interface SamlConfiguration {
  entityId?: string;
  idpMetadata: string;
  usernameAttribute: string;
  firstNameAttribute?: string;
  lastNameAttribute?: string;
  emailAttribute?: string;
  groupsAttribute?: string;
  // Tri-state: null = Default (backend decides), true = Force enabled, false = Force disabled
  validateResponseSignature?: boolean | null;
  validateAssertionSignature?: boolean | null;
}

export interface SamlPageProps {
  className?: string;
}


