import { Button, Flex, Spinner } from '@radix-ui/themes';
import { HardDrive } from 'lucide-react';
import { useCallback, useId, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, useInvalidateApi } from '../../../api';
import { useT } from '../../../i18n';
import { EmptyState, ErrorState, Page, useToast } from '../../../ui';
import { WizardBar } from '../../repositories/components/Wizard';
import { CREATE_ADDRESS, configPath, LIST_ADDRESS, storeType, UI_BLOB_STORES, useUiBlobStores } from '../api';
import styles from '../blobstores.module.css';
import { BlobStoreForm, STEPS, type Step } from '../components/BlobStoreForm';
import { type StoreDocument, template } from '../model';
import { wizardCrumbs } from './CreatePickerPage';

const STEP_TITLES: Record<Step, string> = {
  basic: 'blobstores.wizard.basic',
  credentials: 'blobstores.wizard.credentials',
  advanced: 'blobstores.wizard.advanced',
};

export default function CreateBlobStorePage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const params = useParams();
  const type = storeType(params.type ?? '');
  const stores = useUiBlobStores();
  const document = useMemo(() => (type ? template(type) : null), [type]);
  const onSaved = useCallback(
    (saved: StoreDocument) => {
      const name = String(saved.name ?? '');
      toast.success(t('blobstores.create.created', { name }));
      void invalidate(UI_BLOB_STORES);
      navigate(LIST_ADDRESS, { replace: true });
    },
    [invalidate, navigate, t, toast],
  );
  const formId = useId();
  const [step, setStep] = useState<Step>('basic');
  const [ready, setReady] = useState<Record<Step, boolean>>({
    basic: false,
    credentials: true,
    advanced: true,
  });
  const index = STEPS.indexOf(step);
  const crumbs = [
    { label: t('blobstores.crumbs.settings'), to: '/settings' },
    { label: t('blobstores.crumbs.list'), to: LIST_ADDRESS },
    { label: t('blobstores.crumbs.create'), to: CREATE_ADDRESS },
  ];

  if (!type || !document) {
    return (
      <Page title={t('blobstores.edit.unsupported')} breadcrumbs={crumbs}>
        <EmptyState
          icon={HardDrive}
          title={t('blobstores.edit.unsupported')}
          description={t('blobstores.edit.unsupportedText', { type: params.type ?? '' })}
          action={
            <Button asChild variant="soft">
              <Link to={CREATE_ADDRESS}>{t('blobstores.create.back')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }

  const last = index === STEPS.length - 1;
  return (
    <Page
      className={styles.wizardPage}
      title={t('blobstores.create.title')}
      description={t('blobstores.create.description')}
      breadcrumbs={wizardCrumbs(t)}
    >
      <WizardBar
        step={index + 2}
        steps={4}
        onCancel={() => navigate(LIST_ADDRESS)}
        onBack={() => (index === 0 ? navigate(CREATE_ADDRESS) : setStep(STEPS[index - 1] as Step))}
        next={
          last ? (
            <Button key="create" type="submit" form={formId}>
              {t('blobstores.create.submit')}
            </Button>
          ) : (
            <Button
              key="next"
              type="button"
              disabled={!ready[step]}
              onClick={() => setStep(STEPS[index + 1] as Step)}
            >
              {t('repositories.create.next')}
            </Button>
          )
        }
      />
      <div className={styles.body}>
        <h2 className={styles.stepTitle}>{t(STEP_TITLES[step])}</h2>
        {stores.error ? (
          <ErrorState error={stores.error} onRetry={() => void stores.refetch()} />
        ) : stores.isPending ? (
          <Flex justify="center" py="9">
            <Spinner size="3" />
          </Flex>
        ) : (
          <BlobStoreForm
            type={type}
            creating
            document={document}
            stores={stores.data ?? []}
            submitLabel={t('blobstores.create.submit')}
            onSubmit={async (body) => {
              await api.post(configPath(type), body);
            }}
            onSaved={onSaved}
            formId={formId}
            step={step}
            onStepChange={setStep}
            onReadyChange={setReady}
          />
        )}
      </div>
    </Page>
  );
}
