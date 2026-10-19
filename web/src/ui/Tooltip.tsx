import { Tooltip as ThemeTooltip } from '@radix-ui/themes';
import {
  type ComponentProps,
  cloneElement,
  isValidElement,
  type PointerEvent,
  type ReactElement,
  useRef,
  useState,
} from 'react';
import { currentModality } from './inputModality';

type TooltipProps = ComponentProps<typeof ThemeTooltip>;
type Hoverable = {
  onPointerEnter?: (event: PointerEvent<HTMLElement>) => void;
  onPointerLeave?: (event: PointerEvent<HTMLElement>) => void;
};

export function Tooltip({ children, open: controlled, onOpenChange, ...props }: TooltipProps) {
  const hovered = useRef(false);
  const [open, setOpen] = useState(false);
  const change = (next: boolean) => {
    const shown = next && (hovered.current || currentModality() !== 'pointer');
    setOpen(shown);
    onOpenChange?.(shown);
  };
  const trigger = isValidElement<Hoverable>(children)
    ? cloneElement(children as ReactElement<Hoverable>, {
        onPointerEnter: (event: PointerEvent<HTMLElement>) => {
          hovered.current = true;
          children.props.onPointerEnter?.(event);
        },
        onPointerLeave: (event: PointerEvent<HTMLElement>) => {
          hovered.current = false;
          children.props.onPointerLeave?.(event);
        },
      })
    : children;
  return (
    <ThemeTooltip {...props} open={controlled ?? open} onOpenChange={change}>
      {trigger}
    </ThemeTooltip>
  );
}
