import { Button } from '@radix-ui/themes';
import { AppWindow, Asterisk, Eye, Filter, type LucideIcon, Shield } from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router';
import { useT } from '../../../i18n';
import { Page } from '../../../ui';
import type { PrivilegeTypeId } from '../api';
import { EditBar, EditForm, Progress, StepTitle } from '../shared/Edit';
import { PRIVILEGES_PAGE } from '../shared/links';
import { typeDescription, typeLabel } from './labels';
import styles from './PrivilegeForm.module.css';

export const TYPE_CARDS: { id: PrivilegeTypeId; icon: LucideIcon; tone: string }[] = [
  { id: 'application', icon: AppWindow, tone: 'purple' },
  { id: 'repository-admin', icon: Shield, tone: 'red' },
  { id: 'repository-content-selector', icon: Filter, tone: 'green' },
  { id: 'repository-view', icon: Eye, tone: 'violet' },
  { id: 'wildcard', icon: Asterisk, tone: 'orange' },
];

export function createHref(type: string): string {
  return `${PRIVILEGES_PAGE}/create/${type}`;
}

export default function PrivilegeTypePage() {
  const t = useT();
  const navigate = useNavigate();
  const [chosen, setChosen] = useState<PrivilegeTypeId | null>(null);
  return (
    <Page
      gutter="small"
      title={t('security.privilege.createTitle')}
      description={t('security.privilege.stepType')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.privileges.title'), to: PRIVILEGES_PAGE },
        { label: t('security.privilege.create') },
      ]}
    >
      <EditForm
        id="privilege-type"
        onSubmit={(event) => {
          event.preventDefault();
          if (chosen) {
            navigate(createHref(chosen));
          }
        }}
        bar={
          <EditBar
            start={
              <Button type="button" variant="outline" onClick={() => navigate(PRIVILEGES_PAGE)}>
                {t('security.form.cancel')}
              </Button>
            }
            end={
              <Button type="submit" disabled={!chosen}>
                {t('security.wizard.continue')}
              </Button>
            }
            progress={<Progress total={3} done={1} label={t('security.wizard.progress')} />}
          />
        }
      >
        <StepTitle>{t('security.privilege.chooseType')}</StepTitle>
        <div className={styles.cards} role="radiogroup" aria-label={t('security.privilege.chooseType')}>
          {TYPE_CARDS.map((card) => {
            const Icon = card.icon;
            const selected = chosen === card.id;
            return (
              <button
                key={card.id}
                type="button"
                role="radio"
                aria-checked={selected}
                className={selected ? `${styles.card} ${styles.selected}` : styles.card}
                onClick={() => setChosen(card.id)}
                onDoubleClick={() => navigate(createHref(card.id))}
              >
                <span className={styles.cardInner}>
                  <span className={styles.icon} data-tone={card.tone}>
                    <Icon size={24} aria-hidden="true" />
                  </span>
                  <span className={styles.cardText}>
                    <span className={styles.cardTitle}>{typeLabel(t, card.id)}</span>
                    <span className={styles.cardDescription}>{typeDescription(t, card.id)}</span>
                  </span>
                </span>
              </button>
            );
          })}
        </div>
      </EditForm>
    </Page>
  );
}
