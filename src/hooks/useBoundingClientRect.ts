import { useLayoutEffect, useState } from 'react';

export function useBoundingClientRect(element: Element | null): DOMRect | null {
    const [rect, setRect] = useState(null);

    useLayoutEffect(() => {
        if (element === null) {
            return;
        }

        const observer = new ResizeObserver(() => {
            setRect(element.getBoundingClientRect());
        });
        observer.observe(element);
        return () => {
            observer.disconnect();
        };
    }, [element]);

    return rect;
}
