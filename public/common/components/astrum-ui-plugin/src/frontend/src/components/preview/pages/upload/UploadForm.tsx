

import React, { useCallback, useMemo, useState } from 'react';
import { Badge, Box, Flex, Text } from '@radix-ui/themes';
import { CheckCircle2, Clock } from 'lucide-react';
import { SettingsForm, SettingsFormSection } from '../../shared/form';
import { useUploadForm } from './hooks/useUploadForm';
import { useToast, useUnsavedChangesWarning, clearDirtyState } from '../../shared';
import { RepositorySelector } from './RepositorySelector';
import { FileUploadZone } from './FileUploadZone';
import { UploadFieldRenderer } from './UploadFieldRenderer';
import type {
  RepositorySettings,
  UploadableRepository,
  UploadComponentField,
  UploadFieldDefinition,
  UploadDefinitionExtended,
} from './upload.types';
import UploadStrings from '../../constants/pages/upload/UploadStrings';
import plural from '../../../../interface/plural';

const UPLOAD_FORM_DIRTY_ID = 'upload-form';
const MAVEN_FORMAT = 'maven2';
const MAVEN_COMPONENT_COORDS_GROUP = 'Component coordinates';
const MAVEN_GENERATE_POM_FIELD_NAME = 'generate-pom';
const MAVEN_PACKAGING_FIELD_NAME = 'packaging';
interface UploadFormProps {
  repositoryName: string;
  repositorySettings: RepositorySettings | null;
  componentFields: UploadComponentField[];
  componentFieldsByGroup: Record<string, UploadComponentField[]>;
  assetFields: UploadFieldDefinition[];
  multipleUpload: boolean;
  regexMap: UploadDefinitionExtended['regexMap'] | null;
  availableRepositories?: UploadableRepository[];
  onRepositoryChange?: (repoName: string) => void;
  onBack: () => void;
}
/**
 * UploadForm component for managing the upload of components and assets.
 */
export function UploadForm({
  repositoryName,
  repositorySettings,
  componentFields,
  componentFieldsByGroup,
  assetFields,
  multipleUpload,
  regexMap,
  availableRepositories = [],
  onRepositoryChange,
  onBack,
}: UploadFormProps): JSX.Element {
  const toast = useToast();
  const [localDisabledFields, setLocalDisabledFields] = useState<Set<string>>(new Set());
  const [submitError, setSubmitError] = useState<string | null>(null);
  const {
    formData,
    validationErrors,
    isSubmitting,
    isValid,
    isDirty,
    setAssetFile,
    setAssetField,
    addAsset,
    removeAsset,
    setComponentField,
    blurAssetField,
    submit,
  } = useUploadForm({
    repositoryName,
    componentFields,
    assetFields,
    multipleUpload,
    regexMap,
    disabledFields: localDisabledFields,
  });
  // Register the form's dirty state with the global unsaved-changes guard so the
  // router's onBefore hook prompts before navigating away with pending edits.
  // SettingsForm keeps noDirtyTracking, so this is the single registration path.
  useUnsavedChangesWarning(isDirty, UPLOAD_FORM_DIRTY_ID);
  const hasPom = useMemo(
    () => formData.assets.some((a) => a.file?.name.toLowerCase().endsWith('.pom')),
    [formData.assets],
  );
  useMemo(() => {
    if (repositorySettings?.format !== MAVEN_FORMAT) {
      setLocalDisabledFields(new Set());
      return;
    }
    const disabled = new Set<string>();
    if (hasPom) {
      (componentFieldsByGroup[MAVEN_COMPONENT_COORDS_GROUP] || []).forEach((f) =>
        disabled.add(f.name),
      );
    }
    if (!(formData.componentFields[MAVEN_GENERATE_POM_FIELD_NAME] || hasPom)) {
      disabled.add(MAVEN_PACKAGING_FIELD_NAME);
    }
    setLocalDisabledFields(disabled);
  }, [repositorySettings?.format, hasPom, formData.componentFields, componentFieldsByGroup]);
  const handleSubmit = useCallback(async () => {
    setSubmitError(null);
    const result = await submit();
    if (result.success) {
      toast.success(
        UploadStrings.FORM.COMPONENT_UPLOADED(repositoryName),
        result.componentName ? UploadStrings.FORM.COMPONENT(result.componentName) : undefined,
      );
      // A successful upload leaves formData intact (isDirty stays true), so clear
      // the guard registration synchronously before navigating; the router reads
      // window.dirty during onBack()'s transition and must not prompt.
      clearDirtyState(UPLOAD_FORM_DIRTY_ID);
      onBack();
    } else if (!result.success && result.error) {
      setSubmitError(result.error);
    }
  }, [submit, repositoryName, toast, onBack]);
  const formatDisplay =
    repositorySettings?.format === 'maven2' ? 'Maven' : repositorySettings?.format;
  const filesSelected = formData.assets.filter((a) => a.file).length;
  return (
    <Box className="upload-form">
      <SettingsForm
        title={UploadStrings.FORM.UPLOAD_TO(repositoryName)}
        headerActions={
          formatDisplay && (
            <Badge size="2" variant="soft" color="blue">
              {formatDisplay}
            </Badge>
          )
        }
        onSubmit={handleSubmit}
        onCancel={onBack}
        pristine={false}
        noDirtyTracking
        saving={isSubmitting}
        error={submitError || undefined}
        submitLabel={UploadStrings.FORM.UPLOAD_COMPONENT}
        cancelLabel={UploadStrings.FORM.BACK_UPLOAD}
        submitDisabled={filesSelected === 0}
        testId="upload-form"
        data-valid={isValid ? 'true' : 'false'}
        footerExtra={
          <Flex align="center" gap="2">
            {filesSelected > 0 ? (
              <>
                <CheckCircle2 size={14} className="upload-form__footer-check" />
                <Text size="1" weight="medium">
                  {plural(UploadStrings.FORM.FILES_READY, filesSelected)}
                </Text>
              </>
            ) : (
              <>
                <Clock size={14} />
                <Text size="1" color="gray">{UploadStrings.FORM.NO_FILES_SELECTED}</Text>
              </>
            )}
          </Flex>
        }
      >
        <RepositorySelector
          repositoryName={repositoryName}
          onRepositoryChange={(val) => onRepositoryChange?.(val)}
          availableRepositories={availableRepositories}
          disabled={isSubmitting}
        />
        <Box className="upload-form__format-fields">
        <UploadFieldRenderer
          format={repositorySettings?.format || ''}
          definition={{ componentFields, assetFields, multipleUpload, format: repositorySettings?.format || '', uiUpload: true }}
          fieldsByGroup={componentFieldsByGroup}
          values={formData.componentFields}
          onChange={setComponentField}
          errors={validationErrors.componentFields || {}}
          disabledFields={localDisabledFields}
          hasPomFile={hasPom}
        />
        </Box>
        <SettingsFormSection title={UploadStrings.FORM.ASSETS}>
          <FileUploadZone
            assets={formData.assets}
            assetFields={assetFields}
            validationErrors={validationErrors}
            multipleUpload={multipleUpload}
            format={repositorySettings?.format}
            onFileChange={(idx, files) => setAssetFile(idx, files[0] || null)}
            onAssetFieldChange={setAssetField}
            onBlurAssetField={blurAssetField}
            onAddAsset={addAsset}
            onRemoveAsset={removeAsset}
          />
        </SettingsFormSection>
      </SettingsForm>
    </Box>
  );
}

export default UploadForm;
