

import React, { useMemo } from 'react';

import { SettingsCombobox, SettingsFormSection } from '../../shared/form';
import type { UploadableRepository } from './upload.types';
import UploadStrings from '../../constants/pages/upload/UploadStrings';

interface RepositorySelectorProps {
  repositoryName: string;
  onRepositoryChange: (repoName: string) => void;
  availableRepositories: UploadableRepository[];
  disabled?: boolean;
}

/**
 * Component for selecting a target repository for upload.
 *
 * @param {RepositorySelectorProps} props - The props for the component.
 * @returns {JSX.Element} The rendered component.
 */
export function RepositorySelector({
  repositoryName,
  onRepositoryChange,
  availableRepositories,
  disabled = false,
}: RepositorySelectorProps): JSX.Element {
  const repositoryOptions = useMemo(
    () =>
      availableRepositories.map((repo) => ({
        value: repo.name,
        label: repo.name,
        description: repo.format,
      })),
    [availableRepositories],
  );

  return (
    <SettingsFormSection title={UploadStrings.REPOSITORY_SELECTOR.REPOSITORY}>
      <SettingsCombobox
        name="repository"
        label={UploadStrings.REPOSITORY_SELECTOR.TARGET_REPOSITORY}
        value={repositoryName}
        onChange={(val) => onRepositoryChange(val)}
        options={repositoryOptions}
        helpText={UploadStrings.REPOSITORY_SELECTOR.HOSTED_REPOSITORY_WHERE_COMPONENT_UPLOADED}
        required
        allowCustom={false}
        disabled={disabled}
        aria-label={UploadStrings.REPOSITORY_SELECTOR.SELECT_TARGET_REPOSITORY_ARIA}
      />
    </SettingsFormSection>
  );
}

export default RepositorySelector;
