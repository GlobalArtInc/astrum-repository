import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';

export function useLeave(): { leaving: boolean; leave(to: string): void } {
  const navigate = useNavigate();
  const [target, setTarget] = useState<string | null>(null);
  useEffect(() => {
    if (target !== null) {
      navigate(target);
    }
  }, [target, navigate]);
  return { leaving: target !== null, leave: setTarget };
}
