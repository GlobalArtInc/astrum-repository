import { Badge, Button, Callout, Heading, IconButton, Separator, Spinner, Text } from '@radix-ui/themes';
import {
  ChevronRight,
  CircleAlert,
  CircleCheck,
  Clock,
  File as FileIcon,
  Info,
  Package,
  Plus,
  X,
} from 'lucide-react';
import { type FormEvent, type ReactNode, useEffect, useRef, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { ApiError, useInvalidateApi } from '../../api';
import { formatBytes, useI18n } from '../../i18n';
import { describeError, formatName, UnsavedChangesGuard, useDocumentTitle, useToast } from '../../ui';
import {
  ACCEPTED,
  type AssetDraft,
  assetFieldName,
  assetFileName,
  assetTextFields,
  BOOLEAN,
  componentFieldName,
  emptyAsset,
  emptyDraft,
  fileField,
  hasPom,
  knownFieldNames,
  type Problems,
  placeServerErrors,
  showsRequired,
  suggestAssetValues,
  toFormData,
  type UploadDraft,
  validateDraft,
} from './form';
import { fieldHint, fieldLabel, fieldPlaceholder, PACKAGING, sectionTitle } from './labels';
import type { UploadFieldSpec, UploadRepository, UploadSpec } from './model';
import { useUploadTargets } from './targets';
import { type Progress as UploadProgress, uploadComponent } from './transport';
import styles from './Upload.module.css';
import { Dropzone, SettingsCheckbox, SettingsField, SettingsSelect, TargetCombobox } from './UploadFields';

function badgeText(format: string): string {
  return format === 'maven2' ? formatName(format) : format;
}

function componentSummary(spec: UploadSpec, draft: UploadDraft): string {
  const parts: string[] = [];
  for (const field of spec.componentFields) {
    if (field.type === BOOLEAN || field.name === 'packaging') {
      continue;
    }
    const value = (draft.component[field.name] ?? '').trim();
    if (value !== '') {
      parts.push(...value.split('/').filter((part) => part !== ''));
    }
  }
  if (parts.length === 0) {
    const first = draft.assets.find((asset) => asset.file)?.file;
    return first?.name ?? '';
  }
  return parts.join(' ');
}

function Crumbs({ repository }: { repository: string }) {
  const { t } = useI18n();
  return (
    <nav className={styles.crumbs} aria-label={t('ui.page.breadcrumbs')}>
      <Button asChild variant="ghost" size="2" className={styles.crumbLink}>
        <Link to="/upload">{t('upload.list.title')}</Link>
      </Button>
      <ChevronRight size={14} aria-hidden="true" />
      <Text size="2" weight="medium" className={styles.gray} aria-current="page">
        {repository}
      </Text>
    </nav>
  );
}

function Section({
  title,
  icon,
  id,
  children,
}: {
  title: string;
  icon?: ReactNode;
  id: string;
  children: ReactNode;
}) {
  return (
    <section className={styles.section} aria-labelledby={id}>
      <div className={styles.sectionHead}>
        {icon && <div className={styles.sectionIcon}>{icon}</div>}
        <Heading as="h2" size="3" weight="medium" id={id}>
          {title}
        </Heading>
      </div>
      <div className={styles.sectionContent}>{children}</div>
    </section>
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
  const [finished, setFinished] = useState(false);
  const abort = useRef<AbortController | null>(null);
  const formRef = useRef<HTMLFormElement>(null);

  useEffect(() => {
    if (finished) {
      navigate('/upload');
    }
  }, [finished, navigate]);
  const pom = hasPom(spec, draft);
  const files = fileField(spec);
  const chosen = draft.assets.filter((asset) => asset.file !== null);
  const totalBytes = chosen.reduce((sum, asset) => sum + (asset.file?.size ?? 0), 0);
  const accepted = ACCEPTED[spec.format];
  const generatePom = draft.component['generate-pom'] === 'true';
  const title = t('upload.form.title', { repository: repository.name });
  useDocumentTitle(title);

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
    if (uploading || chosen.length === 0) {
      return;
    }
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
      const summary = componentSummary(spec, sent);
      toast.success(
        t('upload.form.uploaded', { repository: repository.name }),
        summary ? t('upload.form.uploadedComponent', { name: summary }) : undefined,
      );
      void invalidate('/service/rest/v1/search');
      void invalidate('/service/rest/internal/ui');
      setDraft(emptyDraft(spec));
      setFinished(true);
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

  const textField = (
    field: UploadFieldSpec,
    name: string,
    value: string,
    onChange: (value: string) => void,
  ) => (
    <SettingsField
      key={field.name}
      name={name}
      label={fieldLabel(t, spec.format, field)}
      help={fieldHint(t, spec.format, field)}
      placeholder={fieldPlaceholder(t, spec.format, field)}
      error={errors[name]}
      required={showsRequired(field)}
      disabled={uploading}
      value={value}
      onChange={onChange}
    />
  );

  const componentField = (field: UploadFieldSpec) => {
    const name = componentFieldName(spec, field.name);
    const value = draft.component[field.name] ?? '';
    if (field.type === BOOLEAN) {
      return (
        <SettingsCheckbox
          key={field.name}
          label={fieldLabel(t, spec.format, field)}
          description={fieldHint(t, spec.format, field)}
          checked={value === 'true'}
          disabled={uploading || pom}
          onChange={(checked) => setComponent(field.name, checked ? 'true' : 'false')}
        />
      );
    }
    if (spec.format === 'maven2' && field.name === 'packaging') {
      return (
        <SettingsSelect
          key={field.name}
          label={fieldLabel(t, spec.format, field)}
          help={fieldHint(t, spec.format, field)}
          disabled={uploading || !generatePom || pom}
          value={value || PACKAGING[0] || ''}
          options={PACKAGING}
          onChange={(next) => setComponent(field.name, next)}
        />
      );
    }
    return textField(field, name, value, (next) => setComponent(field.name, next));
  };

  const plainFields = spec.componentFields.filter((field) => field.type !== BOOLEAN);
  const checkFields = spec.componentFields.filter((field) => field.type === BOOLEAN);

  let status: ReactNode;
  if (uploading) {
    status = (
      <span className={styles.statusItem} role="status">
        <Spinner size="1" />
        <Text size="1" weight="medium">
          {t('upload.form.uploading', {
            percent,
            sent: formatBytes(language, progress?.loaded ?? 0),
            total: formatBytes(language, progress?.total ?? totalBytes),
          })}
        </Text>
      </span>
    );
  } else if (chosen.length === 0) {
    status = (
      <span className={styles.statusItem}>
        <Clock size={14} aria-hidden="true" />
        <Text size="1" className={styles.gray}>
          {t('upload.form.noFiles')}
        </Text>
      </span>
    );
  } else {
    status = (
      <span className={styles.statusItem}>
        <CircleCheck size={14} aria-hidden="true" className={styles.ready} />
        <Text size="1" weight="medium">
          {t('upload.form.ready', { count: chosen.length })}
        </Text>
      </span>
    );
  }

  return (
    <div className={styles.formPage}>
      <UnsavedChangesGuard when={!finished && (uploading || chosen.length > 0)} />
      <Crumbs repository={repository.name} />
      <div className={styles.formOuter}>
        <form ref={formRef} className={styles.form} onSubmit={submit} noValidate>
          <header className={styles.formHeader}>
            <Heading as="h1" size="6" weight="medium">
              {title}
            </Heading>
            <Badge size="2" className={styles.formatBadge}>
              {badgeText(spec.format)}
            </Badge>
          </header>
          <div className={styles.bar}>
            <div className={styles.status}>
              {status}
              {formError && (
                <span className={`${styles.statusItem} ${styles.statusError}`} role="alert">
                  <CircleAlert size={14} aria-hidden="true" className={styles.danger} />
                  <Text size="1" weight="medium" className={styles.danger}>
                    {formError}
                  </Text>
                </span>
              )}
            </div>
            <div className={styles.barActions}>
              {uploading ? (
                <Button type="button" variant="outline" onClick={() => abort.current?.abort()}>
                  {t('upload.form.cancel')}
                </Button>
              ) : (
                <Button asChild variant="outline">
                  <Link to="/upload">{t('upload.form.back')}</Link>
                </Button>
              )}
              <Button type="submit" loading={uploading} disabled={chosen.length === 0}>
                {t('upload.form.submit')}
              </Button>
            </div>
          </div>
          <div className={styles.content}>
            <div className={styles.inner}>
              <Section title={t('upload.form.repository')} id="upload-repository">
                <TargetCombobox
                  label={t('upload.form.target')}
                  help={t('upload.form.targetHint')}
                  placeholder={t('upload.form.targetPlaceholder')}
                  ariaLabel={t('upload.form.targetLabel')}
                  value={repository.name}
                  disabled={uploading}
                  options={targets.map((target) => ({ value: target.name, description: target.format }))}
                  onSelect={(name) => navigate(`/upload/${encodeURIComponent(name)}`)}
                />
              </Section>
              <Separator size="4" className={styles.separator} />
              {spec.componentFields.length > 0 && (
                <>
                  <Section
                    title={sectionTitle(t, spec.format)}
                    id="upload-component"
                    icon={
                      spec.format === 'raw' ? (
                        <FileIcon size={18} aria-hidden="true" />
                      ) : (
                        <Package size={18} aria-hidden="true" />
                      )
                    }
                  >
                    {spec.format === 'maven2' && pom && (
                      <Callout.Root color="blue" size="1">
                        <Callout.Icon>
                          <Info size={16} />
                        </Callout.Icon>
                        <Callout.Text>{t('upload.form.fromPom')}</Callout.Text>
                      </Callout.Root>
                    )}
                    <div className={spec.format === 'maven2' ? styles.grid : styles.stack}>
                      {plainFields.map(componentField)}
                    </div>
                    {checkFields.map(componentField)}
                  </Section>
                  <Separator size="4" className={styles.separator} />
                </>
              )}
              <Section title={t('upload.form.assets')} id="upload-assets">
                <div>
                  {draft.assets.map((asset, index) => {
                    const fileName = assetFileName(spec, index);
                    const number = index + 1;
                    const textFields = assetTextFields(spec);
                    return (
                      <div
                        key={asset.key}
                        className={styles.asset}
                        role="group"
                        aria-label={t('upload.form.assetNumber', { number })}
                      >
                        {draft.assets.length > 1 && (
                          <div className={styles.assetHead}>
                            <div className={styles.assetTitle}>
                              <span className={styles.assetNumber}>{number}</span>
                              <Text size="2" weight="medium">
                                {t('upload.form.assetNumber', { number })}
                              </Text>
                            </div>
                            <IconButton
                              type="button"
                              size="1"
                              variant="ghost"
                              color="red"
                              disabled={uploading}
                              aria-label={t('upload.form.removeAsset')}
                              onClick={() =>
                                setDraft((current) => ({
                                  ...current,
                                  assets: current.assets.filter((candidate) => candidate.key !== asset.key),
                                }))
                              }
                            >
                              <X size={14} aria-hidden="true" />
                            </IconButton>
                          </div>
                        )}
                        {files && (
                          <Dropzone
                            label={fieldLabel(t, spec.format, files)}
                            help={t('upload.file.help')}
                            name={fileName}
                            file={asset.file}
                            accept={accepted?.join(',')}
                            disabled={uploading}
                            error={errors[fileName]}
                            onChange={(file) => {
                              clearError(fileName);
                              setAsset(asset.key, (current, whole) => ({
                                ...current,
                                file,
                                values: file
                                  ? suggestAssetValues(spec, whole, current, file)
                                  : current.values,
                              }));
                            }}
                          />
                        )}
                        {textFields.length > 0 && (
                          <div className={styles.assetFields}>
                            <Text as="p" size="1" className={styles.describe}>
                              {t('upload.form.describeAsset')}
                            </Text>
                            <div className={styles.stack}>
                              {textFields.map((field) => {
                                const name = assetFieldName(spec, index, field.name);
                                return textField(field, name, asset.values[field.name] ?? '', (value) => {
                                  clearError(name);
                                  setAsset(asset.key, (current) => ({
                                    ...current,
                                    values: { ...current.values, [field.name]: value },
                                  }));
                                });
                              })}
                            </div>
                          </div>
                        )}
                      </div>
                    );
                  })}
                  {spec.multipleUpload && (
                    <Button
                      type="button"
                      variant="soft"
                      className={styles.addAsset}
                      disabled={uploading}
                      aria-label={t('upload.form.addAsset')}
                      onClick={() =>
                        setDraft((current) => ({ ...current, assets: [...current.assets, emptyAsset(spec)] }))
                      }
                    >
                      <Plus size={16} aria-hidden="true" />
                      {t('upload.form.addAsset')}
                    </Button>
                  )}
                </div>
              </Section>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}

function Unavailable({ name, children }: { name: string; children: ReactNode }) {
  return (
    <div className={styles.formPage}>
      <Crumbs repository={name} />
      <div className={styles.formOuter}>{children}</div>
    </div>
  );
}

export default function UploadPage() {
  const { t } = useI18n();
  const { repository: name = '' } = useParams();
  const { targets, specs, loading, error, retry } = useUploadTargets();
  const target = targets.find((candidate) => candidate.name === name);
  const spec = specs?.find((candidate) => candidate.format === target?.format);
  useDocumentTitle(t('upload.form.title', { repository: name }));
  if (loading) {
    return (
      <Unavailable name={name}>
        <div className={styles.loading} role="status">
          <Spinner size="3" />
          <Text size="2" className={styles.gray}>
            {t('upload.form.loading')}
          </Text>
        </div>
      </Unavailable>
    );
  }
  if (error) {
    return (
      <Unavailable name={name}>
        <Callout.Root color="red" role="alert">
          <Callout.Icon>
            <CircleAlert size={16} />
          </Callout.Icon>
          <Callout.Text>{describeError(t, error)}</Callout.Text>
        </Callout.Root>
        <Button type="button" variant="soft" color="gray" onClick={retry} className={styles.retry}>
          {t('upload.form.retry')}
        </Button>
      </Unavailable>
    );
  }
  if (!target || !spec) {
    return (
      <Unavailable name={name}>
        <Callout.Root color="red" role="alert">
          <Callout.Icon>
            <CircleAlert size={16} />
          </Callout.Icon>
          <Callout.Text>{t('upload.form.unavailableText', { repository: name })}</Callout.Text>
        </Callout.Root>
        <Button asChild variant="outline" className={styles.retry}>
          <Link to="/upload">{t('upload.form.back')}</Link>
        </Button>
      </Unavailable>
    );
  }
  return <UploadForm key={target.name} repository={target} spec={spec} targets={targets} />;
}
