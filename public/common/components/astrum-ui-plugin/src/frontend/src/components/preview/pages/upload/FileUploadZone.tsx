

import React, { useState } from 'react';
import { Box, Button, Callout, Flex, IconButton, Separator, Text } from '@radix-ui/themes';
import { AlertTriangle, Info, Plus, Trash2 } from 'lucide-react';

import { FileDropzone } from './components/FileDropzone';
import { UploadField } from './UploadField';
import type { AssetFieldData, UploadFieldDefinition, ValidationErrors } from './upload.types';
import { UPLOAD_FORM_STRINGS } from './upload.types';
import UploadStrings from '../../constants/pages/upload/UploadStrings';

interface FileUploadZoneProps {
  assets: AssetFieldData[];
  assetFields: UploadFieldDefinition[];
  validationErrors: ValidationErrors;
  multipleUpload: boolean;
  format?: string;
  onFileChange: (assetIndex: number, files: File[]) => void;
  onAssetFieldChange: (assetIndex: number, fieldName: string, value: string | boolean) => void;
  onBlurAssetField?: (assetIndex: number, fieldName: string) => void;
  onAddAsset: () => void;
  onRemoveAsset: (assetIndex: number) => void;
}

const MAX_ZIP_SIZE = 50 * 1024 * 1024; // 50 MB

async function extractGoModulePath(file: File): Promise<string | null> {
  try {
    // unzipSync avoids blob-URL Web Workers, which Astrum CSP blocks
    const { unzipSync } = await import('fflate');
    const buffer = await file.arrayBuffer();
    const unzipped = unzipSync(new Uint8Array(buffer));
    for (const [filename, bytes] of Object.entries(unzipped)) {
      // Only match go.mod at the module root:
      //   "go.mod"                          — non-prefixed zip
      //   "module@version/go.mod"           — standard Go module proxy format
      // Nested go.mod files (e.g. subpkg/go.mod) are intentionally ignored.
      if (filename !== 'go.mod' && !/^[^@]+@[^/]+\/go\.mod$/.test(filename)) {
        continue;
      }
      const content = new TextDecoder().decode(bytes);
      const moduleLine = content.split('\n').find((line) => line.trim().startsWith('module '));
      if (moduleLine) {
        return moduleLine.trim().replace(/^module\s+/, '').trim();
      }
    }
  } catch {
    // silently ignore extraction errors
  }
  return null;
}

/**
 * FileUploadZone component for managing multiple asset uploads.
 */
export function FileUploadZone({
  assets,
  assetFields,
  validationErrors,
  multipleUpload,
  format,
  onFileChange,
  onAssetFieldChange,
  onBlurAssetField,
  onAddAsset,
  onRemoveAsset,
}: FileUploadZoneProps): JSX.Element {
  type GoZipState = { status: 'too-large' } | { status: 'found'; path: string } | { status: 'not-found' };
  const [goZipStates, setGoZipStates] = useState<Record<number, GoZipState>>({});

  const setGoZip = (assetIndex: number, state: GoZipState) =>
    setGoZipStates((prev) => ({ ...prev, [assetIndex]: state }));

  const clearGoZip = (assetIndex: number) =>
    setGoZipStates((prev) => { const next = { ...prev }; delete next[assetIndex]; return next; });

  const handleFileChange = (assetIndex: number, files: File[]) => {
    onFileChange(assetIndex, files);
    if (format === 'go' && files.length > 0 && files[0].name.endsWith('.zip')) {
      const file = files[0];
      if (file.size > MAX_ZIP_SIZE) {
        setGoZip(assetIndex, { status: 'too-large' });
      } else {
        clearGoZip(assetIndex); // clear stale state before async extraction
        extractGoModulePath(file)
          .then((modulePath) => {
            setGoZip(assetIndex, modulePath ? { status: 'found', path: modulePath } : { status: 'not-found' });
          })
          .catch(() => {
            setGoZip(assetIndex, { status: 'not-found' });
          });
      }
    } else {
      clearGoZip(assetIndex);
    }
  };

  return (
    <Box data-testid="file-upload-zone">
      {assets.map((asset, assetIndex) => (
        <Box key={assetIndex} className="upload-form-page__asset" mb="4">
          {assets.length > 1 && (
            <Flex justify="between" align="center" mb="3" className="upload-form-page__asset-header">
              <Flex align="center" gap="2">
                <Box className="upload-form-page__asset-number">{assetIndex + 1}</Box>
                <Text size="2" weight="medium">{UploadStrings.FILE_UPLOAD_ZONE.ASSET(assetIndex + 1)}</Text>
              </Flex>
              <IconButton
                type="button"
                variant="ghost"
                color="red"
                size="1"
                onClick={() => onRemoveAsset(assetIndex)}
                aria-label={UPLOAD_FORM_STRINGS.removeAsset}
              >
                <Trash2 size={14} />
              </IconButton>
            </Flex>
          )}

          <FileDropzone
            files={asset.file ? [asset.file] : []}
            onChange={(files) => handleFileChange(assetIndex, files)}
            label={UploadStrings.FILE_UPLOAD_ZONE.FILE}
            helpText={
              assetFields.find((f) => f.type === 'FILE')?.helpText ??
              UploadStrings.FILE_UPLOAD_ZONE.ARTIFACT_ASSET_FILE_UPLOAD_DRAG
            }
            required
            error={validationErrors.assets?.[assetIndex]?.file ?? undefined}
            id={`asset-file-${assetIndex}`}
            testId={`input-asset-file-${assetIndex}`}
          />

          {format === 'go' && goZipStates[assetIndex]?.status === 'too-large' && (
            <Callout.Root size="1" color="yellow" mt="2" data-testid={`go-module-path-large-${assetIndex}`}>
              <Callout.Icon>
                <AlertTriangle size={14} />
              </Callout.Icon>
              <Callout.Text>
                {UploadStrings.FILE_UPLOAD_ZONE.FILE_EXCEEDS_50_MB_MODULE}
              </Callout.Text>
            </Callout.Root>
          )}

          {format === 'go' && goZipStates[assetIndex]?.status === 'not-found' && (
            <Callout.Root size="1" color="yellow" mt="2" data-testid={`go-module-no-mod-${assetIndex}`}>
              <Callout.Icon>
                <AlertTriangle size={14} />
              </Callout.Icon>
              <Callout.Text>
                {UploadStrings.FILE_UPLOAD_ZONE.NO_GO_MODULE_PATH(<strong>module</strong>)}
              </Callout.Text>
            </Callout.Root>
          )}

          {format === 'go' && goZipStates[assetIndex]?.status === 'found' && (
            <Callout.Root size="1" color="blue" mt="2" data-testid={`go-module-path-${assetIndex}`}>
              <Callout.Icon>
                <Info size={14} />
              </Callout.Icon>
              <Callout.Text>
                {UploadStrings.FILE_UPLOAD_ZONE.GO_MODULE_PATH(<strong>{(goZipStates[assetIndex] as { status: 'found'; path: string }).path}</strong>)}
              </Callout.Text>
            </Callout.Root>
          )}

          {assetFields.filter((f) => f.type !== 'FILE').length > 0 && (
            <Box mt="4">
              <Text as="p" size="1" color="gray" mb="3">
                {UploadStrings.FILE_UPLOAD_ZONE.SPECIFY_ASSET_METADATA}
              </Text>
              <Flex direction="column" gap="3" data-testid={`asset-fields-${assetIndex}`}>
                {assetFields
                  .filter((field) => field.type !== 'FILE')
                  .map((field) => (
                    <UploadField
                      key={field.name}
                      field={field}
                      value={asset[field.name] as string | boolean}
                      onChange={(val) => onAssetFieldChange(assetIndex, field.name, val)}
                      error={validationErrors.assets?.[assetIndex]?.[field.name]}
                      onBlur={onBlurAssetField ? () => onBlurAssetField(assetIndex, field.name) : undefined}
                    />
                  ))}
              </Flex>
            </Box>
          )}

          {assetIndex < assets.length - 1 && (
            <Separator my="4" size="4" className="upload-form-page__asset-separator" />
          )}
        </Box>
      ))}

      {multipleUpload && (
        <Button
          type="button"
          variant="soft"
          onClick={onAddAsset}
          mt="2"
          className="upload-form-page__add-asset"
          aria-label={UploadStrings.FILE_UPLOAD_ZONE.ADD_ANOTHER_ASSET_ARIA}
        >
          <Plus size={16} />
          {UploadStrings.FILE_UPLOAD_ZONE.ADD_ANOTHER_ASSET}
        </Button>
      )}
    </Box>
  );
}

export default FileUploadZone;
