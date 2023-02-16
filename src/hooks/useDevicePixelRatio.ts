import { useEffect, useState } from 'react';

export function useDevicePixelRatio(): number {
    const [devicePixelRatio, setDevicePixelRatio] = useState(
        window.devicePixelRatio,
    );
    useEffect(() => {
        const media = window.matchMedia(
            `(resolution: ${devicePixelRatio}dppx)`,
        );
        const handleChange = () => {
            setDevicePixelRatio(window.devicePixelRatio);
        };

        media.addEventListener('change', handleChange);
        return () => {
            media.removeEventListener('change', handleChange);
        };
    }, [devicePixelRatio]);
    return devicePixelRatio;
}
