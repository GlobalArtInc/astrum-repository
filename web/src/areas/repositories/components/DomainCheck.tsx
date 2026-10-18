import { Callout } from '@radix-ui/themes';
import { CircleCheck, Info, TriangleAlert } from 'lucide-react';
import { useEffect, useState } from 'react';
import { api, INTERNAL, withQuery } from '../../../api';
import { type Translate, useT } from '../../../i18n';
import { domainHost, isDomainHost } from '../model/form';

export const DOMAIN_CHECK = `${INTERNAL}/ui/domains/check`;

export interface DomainCheck {
  host: string;
  status: 'invalid' | 'missing' | 'unknown' | 'same' | 'other';
  addresses: string[];
  serverAddresses: string[];
}

export async function checkDomain(host: string): Promise<DomainCheck | null> {
  try {
    return await api.get<DomainCheck>(withQuery(DOMAIN_CHECK, { host }));
  } catch {
    return null;
  }
}

export function domainWarning(check: DomainCheck | null, t: Translate): string | null {
  if (check?.status === 'missing') {
    return t('repositories.form.domainCheck.missing', { host: check.host });
  }
  if (check?.status === 'other') {
    return t('repositories.form.domainCheck.other', {
      host: check.host,
      addresses: check.addresses.join(', '),
      server: check.serverAddresses.join(', '),
    });
  }
  return null;
}

export function useDomainCheck(host: string): DomainCheck | null {
  const [check, setCheck] = useState<DomainCheck | null>(null);
  const normalized = isDomainHost(host) ? domainHost(host) : '';
  useEffect(() => {
    if (normalized === '') {
      return;
    }
    let live = true;
    const timer = window.setTimeout(() => {
      void checkDomain(normalized).then((found) => {
        if (live) {
          setCheck(found);
        }
      });
    }, 600);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [normalized]);
  return check !== null && check.host === normalized ? check : null;
}

export function DomainStatus({ check }: { check: DomainCheck | null }) {
  const t = useT();
  if (check === null || check.status === 'invalid') {
    return null;
  }
  const warning = domainWarning(check, t);
  if (warning) {
    return (
      <Callout.Root color="amber" size="1" role="status">
        <Callout.Icon>
          <TriangleAlert size={16} />
        </Callout.Icon>
        <Callout.Text>{warning}</Callout.Text>
      </Callout.Root>
    );
  }
  const same = check.status === 'same';
  return (
    <Callout.Root color={same ? 'green' : 'gray'} size="1" role="status">
      <Callout.Icon>{same ? <CircleCheck size={16} /> : <Info size={16} />}</Callout.Icon>
      <Callout.Text>
        {t(same ? 'repositories.form.domainCheck.same' : 'repositories.form.domainCheck.unknown', {
          host: check.host,
          addresses: check.addresses.join(', '),
        })}
      </Callout.Text>
    </Callout.Root>
  );
}
