import { Button, Callout, Flex, Heading, IconButton, Progress, Spinner, Text } from '@radix-ui/themes';
import { CircleCheck, Info, Package, Plus, Trash2, Upload } from 'lucide-react';
import { type FormEvent, useRef, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { ApiError, api, useInvalidateApi, withQuery } from '../../api';
import { formatBytes, useI18n } from '../../i18n';
import {
  CheckboxField,
  describeError,
  EmptyState,
  ErrorState,
  FormatBadge,
  FormError,
  Page,
  SelectField,
  TextField,
  UnsavedChangesGuard,
  useToast,
} from '../../ui';
import {
  browsePath,
  componentPath,
  folderOf,
  SEARCH,
  type SearchComponent,
  type SearchPage,
} from '../search/model';
import { FilePicker } from './FilePicker';
import {
  ACCEPTED,
  type AssetDraft,
  assetFieldName,
  assetFileName,
  assetTextFields,
  BOOLEAN,
  componentFieldName,
  componentGroups,
  emptyAsset,
  emptyDraft,
  fileField,
  folderGuess,
  hasPom,
  isRequired,
  knownFieldNames,
  optionValues,
  type Problems,
  placeServerErrors,
  suggestAssetValues,
  toFormData,
  type UploadDraft,
  validateDraft,
} from './form';
import { fieldHint, fieldLabel, formatIntro, groupTitle } from './labels';
import type { UploadFieldSpec, UploadRepository, UploadSpec } from './model';
import { useUploadTargets } from './targets';
import { sha1Of, type Progress as UploadProgress, uploadComponent } from './transport';
import styles from './Upload.module.css';

interface Done {
  repository: string;
  folder: string;
  component: SearchComponent | null;
}

async function findUploaded(repository: string, spec: UploadSpec, draft: UploadDraft): Promise<Done> {
  const done: Done = { repository, folder: folderGuess(spec, draft), component: null };
  const first = draft.assets.find((asset) => asset.file)?.file;
  const sha1 = first ? await sha1Of(first) : null;
  if (!sha1) {
    return done;
  }
  try {
    const page = await api.get<SearchPage<SearchComponent>>(withQuery(SEARCH, { repository, sha1 }));
    const match = page.items[0];
    if (match) {
      const asset = match.assets.find((candidate) => candidate.checksum?.sha1 === sha1) ?? match.assets[0];
      return { repository, component: match, folder: asset ? folderOf(asset.path) : done.folder };
    }
  } catch {}
  return done;
}

function fieldInput(
  field: UploadFieldSpec,
  props: {
    name: string;
    label: string;
    hint?: string;
    error?: string;
    required: boolean;
    disabled: boolean;
    value: string;
    onChange(value: string): void;
  },
) {
  if (field.type === BOOLEAN) {
    return (
      <CheckboxField
        name={props.name}
        label={props.label}
        description={props.hint}
        error={props.error}
        disabled={props.disabled}
        value={props.value === 'true'}
        onChange={(checked) => props.onChange(checked ? 'true' : 'false')}
      />
    );
  }
  const options = optionValues(field);
  if (options.length > 0) {
    return (
      <SelectField
        name={props.name}
        label={props.label}
        hint={props.hint}
        error={props.error}
        required={props.required}
        disabled={props.disabled}
        value={props.value}
        options={options.map((option) => ({ value: option, label: option }))}
        onChange={props.onChange}
      />
    );
  }
  return (
    <TextField
      name={props.name}
      label={props.label}
      hint={props.hint}
      error={props.error}
      required={props.required}
      disabled={props.disabled}
      value={props.value}
      autoComplete="off"
      onChange={props.onChange}
    />
  );
}

function UploadForm({
  repository,
  spec,
  targets,
}: {
  repository: UploadRepository;
  spec: UploadSpec;
  targets: UploadRepository[];
}) {
  const { t, language } = useI18n();
  const navigate = useNavigate();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const [draft, setDraft] = useState<UploadDraft>(() => emptyDraft(spec));
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const [progress, setProgress] = useState<UploadProgress | null>(null);
  const [done, setDone] = useState<Done | null>(null);
  const abort = useRef<AbortController | null>(null);
  const formRef = useRef<HTMLFormElement>(null);
  const pom = hasPom(spec, draft);
  const files = fileField(spec);
  const chosen = draft.assets.filter((asset) => asset.file !== null);
  const totalBytes = chosen.reduce((sum, asset) => sum + (asset.file?.size ?? 0), 0);
  const accepted = ACCEPTED[spec.format];
  const intro = formatIntro(t, spec.format);

  const translate = (problems: Problems) =>
    Object.fromEntries(
      Object.entries(problems).map(([name, problem]) => [name, t(problem.message, problem.vars)]),
    );

  const clearError = (name: string) => {
    if (errors[name]) {
      const { [name]: _, ...rest } = errors;
      setErrors(rest);
    }
  };

  const setComponent = (field: string, value: string) => {
    setDraft((current) => ({ ...current, component: { ...current.component, [field]: value } }));
    clearError(componentFieldName(spec, field));
  };

  const setAsset = (key: number, change: (asset: AssetDraft, current: UploadDraft) => AssetDraft) => {
    setDraft((current) => ({
      ...current,
      assets: current.assets.map((asset) => (asset.key === key ? change(asset, current) : asset)),
    }));
  };

  const focusFirstProblem = () => {
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
    });
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (uploading) {
      return;
    }
    setDone(null);
    const found = validateDraft(spec, draft);
    if (Object.keys(found).length > 0) {
      setErrors(translate(found));
      setFormError(t('upload.form.fix'));
      focusFirstProblem();
      return;
    }
    setErrors({});
    setFormError(null);
    setUploading(true);
    setProgress({ loaded: 0, total: totalBytes });
    const controller = new AbortController();
    abort.current = controller;
    const sent = draft;
    try {
      await uploadComponent(repository.name, toFormData(spec, sent), setProgress, controller.signal);
      const result = await findUploaded(repository.name, spec, sent);
      setDone(result);
      setDraft(emptyDraft(spec));
      toast.success(t('upload.form.uploaded', { repository: repository.name }));
      void invalidate('/service/rest/v1/search');
      void invalidate('/service/rest/internal/ui');
    } catch (failure) {
      if (failure instanceof DOMException && failure.name === 'AbortError') {
        setFormError(t('upload.form.cancelled'));
      } else if (failure instanceof ApiError && failure.status === 400 && failure.fieldErrors.length > 0) {
        const placed = placeServerErrors(knownFieldNames(spec, sent), failure.fieldErrors);
        setErrors(placed.fields);
        setFormError(placed.general.length > 0 ? placed.general.join(' ') : t('upload.form.fix'));
        focusFirstProblem();
      } else {
        setFormError(describeError(t, failure));
      }
    } finally {
      abort.current = null;
      setUploading(false);
      setProgress(null);
    }
  };

  const percent = progress && progress.total > 0 ? Math.round((progress.loaded / progress.total) * 100) : 0;

  return (
    <Page
      title={t('upload.form.title', { repository: repository.name })}
      description={t('upload.form.description')}
      breadcrumbs={[{ label: t('upload.list.title'), to: '/upload' }, { label: repository.name }]}
      actions={<FormatBadge format={repository.format} />}
    >
      <UnsavedChangesGuard when={uploading || chosen.length > 0} />
      {done && (
        <Callout.Root color="green" role="status">
          <Callout.Icon>
            <CircleCheck size={16} />
          </Callout.Icon>
          <Callout.Text>
            {t('upload.form.uploaded', { repository: done.repository })}
            <span className={styles.links}>
              {done.component && (
                <Link className={styles.link} to={componentPath(done.component)}>
                  {t('upload.form.openComponent', {
                    name: done.component.name,
                    version: done.component.version ?? '',
                  })}
                </Link>
              )}
              <Link className={styles.link} to={browsePath(done.repository, done.folder)}>
                {t('upload.form.openFolder')}
              </Link>
            </span>
          </Callout.Text>
        </Callout.Root>
      )}
      <form ref={formRef} className={styles.form} onSubmit={submit} noValidate>
        <section className={styles.section} aria-labelledby="upload-repository">
          <Heading as="h2" size="4" id="upload-repository">
            {t('upload.form.repository')}
          </Heading>
          <SelectField
            label={t('upload.form.target')}
            hint={t('upload.form.targetHint')}
            value={repository.name}
            disabled={uploading}
            options={targets.map((target) => ({ value: target.name, label: target.name }))}
            onChange={(name) => navigate(`/upload/${encodeURIComponent(name)}`)}
          />
        </section>
        {componentGroups(spec).map((group, index) => {
          const title = groupTitle(t, group.title) ?? t('upload.form.component');
          const headingId = `upload-group-${index}`;
          return (
            <section key={group.title ?? index} className={styles.section} aria-labelledby={headingId}>
              <div className={styles.sectionHead}>
                <Package size={18} aria-hidden="true" color="var(--accent-11)" />
                <Heading as="h2" size="4" id={headingId}>
                  {title}
                </Heading>
              </div>
              {spec.format === 'maven2' && pom && group.title !== null && (
                <Callout.Root color="blue" size="1">
                  <Callout.Icon>
                    <Info size={16} />
                  </Callout.Icon>
                  <Callout.Text>{t('upload.form.fromPom')}</Callout.Text>
                </Callout.Root>
              )}
              <div className={styles.grid}>
                {group.fields.map((field) => {
                  const name = componentFieldName(spec, field.name);
                  return (
                    <div key={field.name} className={field.type === BOOLEAN ? styles.wide : undefined}>
                      {fieldInput(field, {
                        name,
                        label: fieldLabel(t, spec.format, field),
                        hint: fieldHint(t, spec.format, field),
                        error: errors[name],
                        required: isRequired(spec, draft, field),
                        disabled: uploading || (pom && field.name === 'generate-pom'),
                        value: draft.component[field.name] ?? '',
                        onChange: (value) => setComponent(field.name, value),
                      })}
                    </div>
                  );
                })}
              </div>
            </section>
          );
        })}
        <section className={styles.section} aria-labelledby="upload-assets">
          <Flex direction="column" gap="1">
            <Heading as="h2" size="4" id="upload-assets">
              {t('upload.form.assets')}
            </Heading>
            {intro && (
              <Text size="2" color="gray">
                {intro}
              </Text>
            )}
          </Flex>
          {draft.assets.map((asset, index) => {
            const fileName = assetFileName(spec, index);
            const number = index + 1;
            return (
              <div
                key={asset.key}
                className={styles.asset}
                role="group"
                aria-label={t('upload.form.assetNumber', { number })}
              >
                {spec.multipleUpload && (
                  <div className={styles.assetHead}>
                    <Text size="2" weight="medium">
                      {t('upload.form.assetNumber', { number })}
                    </Text>
                    {draft.assets.length > 1 && (
                      <IconButton
                        type="button"
                        size="1"
                        variant="ghost"
                        color="red"
                        disabled={uploading}
                        aria-label={t('upload.form.removeAsset', { number })}
                        onClick={() =>
                          setDraft((current) => ({
                            ...current,
                            assets: current.assets.filter((candidate) => candidate.key !== asset.key),
                          }))
                        }
                      >
                        <Trash2 size={14} aria-hidden="true" />
                      </IconButton>
                    )}
                  </div>
                )}
                {files && (
                  <FilePicker
                    label={fieldLabel(t, spec.format, files)}
                    name={fileName}
                    file={asset.file}
                    accept={accepted?.join(',')}
                    required
                    disabled={uploading}
                    error={errors[fileName]}
                    hint={accepted ? t('upload.file.accepted', { types: accepted.join(', ') }) : undefined}
                    onChange={(file) => {
                      clearError(fileName);
                      setAsset(asset.key, (current, whole) => ({
                        ...current,
                        file,
                        values: file ? suggestAssetValues(spec, whole, current, file) : current.values,
                      }));
                    }}
                  />
                )}
                {assetTextFields(spec).length > 0 && (
                  <div className={styles.grid}>
                    {assetTextFields(spec).map((field) => {
                      const name = assetFieldName(spec, index, field.name);
                      return (
                        <div key={field.name}>
                          {fieldInput(field, {
                            name,
                            label: fieldLabel(t, spec.format, field),
                            hint: fieldHint(t, spec.format, field),
                            error: errors[name],
                            required: isRequired(spec, draft, field),
                            disabled: uploading,
                            value: asset.values[field.name] ?? '',
                            onChange: (value) => {
                              clearError(name);
                              setAsset(asset.key, (current) => ({
                                ...current,
                                values: { ...current.values, [field.name]: value },
                              }));
                            },
                          })}
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
          {spec.multipleUpload && (
            <div>
              <Button
                type="button"
                variant="soft"
                disabled={uploading}
                onClick={() =>
                  setDraft((current) => ({ ...current, assets: [...current.assets, emptyAsset(spec)] }))
                }
              >
                <Plus size={16} aria-hidden="true" />
                {t('upload.form.addAsset')}
              </Button>
            </div>
          )}
        </section>
        <FormError message={formError} />
        <div className={styles.actions}>
          {uploading ? (
            <div className={styles.progress}>
              <Text size="2" role="status">
                {t('upload.form.progress', {
                  percent,
                  sent: formatBytes(language, progress?.loaded ?? 0),
                  total: formatBytes(language, progress?.total ?? totalBytes),
                })}
              </Text>
              <Progress value={percent} max={100} aria-label={t('upload.form.progressLabel')} />
            </div>
          ) : (
            <Text size="2" color="gray">
              {chosen.length === 0
                ? t('upload.form.noFiles')
                : t('upload.form.ready', { count: chosen.length, size: formatBytes(language, totalBytes) })}
            </Text>
          )}
          <Flex gap="2">
            {uploading ? (
              <Button type="button" variant="soft" color="gray" onClick={() => abort.current?.abort()}>
                {t('upload.form.cancel')}
              </Button>
            ) : (
              <Button asChild variant="soft" color="gray">
                <Link to="/upload">{t('upload.form.back')}</Link>
              </Button>
            )}
            <Button type="submit" loading={uploading}>
              <Upload size={16} aria-hidden="true" />
              {t('upload.form.submit')}
            </Button>
          </Flex>
        </div>
      </form>
    </Page>
  );
}

export default function UploadPage() {
  const { t } = useI18n();
  const { repository: name = '' } = useParams();
  const { targets, specs, loading, error, retry } = useUploadTargets();
  const target = targets.find((candidate) => candidate.name === name);
  const spec = specs?.find((candidate) => candidate.format === target?.format);
  const title = t('upload.form.title', { repository: name });
  const crumbs = [{ label: t('upload.list.title'), to: '/upload' }, { label: name }];
  if (loading) {
    return (
      <Page title={title} breadcrumbs={crumbs}>
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      </Page>
    );
  }
  if (error) {
    return (
      <Page title={title} breadcrumbs={crumbs}>
        <ErrorState error={error} onRetry={retry} />
      </Page>
    );
  }
  if (!target || !spec) {
    return (
      <Page title={title} breadcrumbs={crumbs}>
        <EmptyState
          icon={Upload}
          title={t('upload.form.unavailableTitle')}
          description={t('upload.form.unavailableText', { repository: name })}
          action={
            <Button asChild variant="soft">
              <Link to="/upload">{t('upload.form.back')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }
  return <UploadForm key={target.name} repository={target} spec={spec} targets={targets} />;
}
