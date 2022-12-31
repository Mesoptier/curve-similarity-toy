import { useState } from 'react';

import { JsCurve } from '@rs_lib';
import { CurveSpaceView } from './CurveSpaceView';
import { ParamSpaceView } from './ParamSpaceView';

export function App(): JSX.Element {
    const [curves, setCurves] = useState<[JsCurve, JsCurve]>([
        new JsCurve([
            { x: 198, y: 158 },
            { x: 150, y: 248 },
            { x: 91, y: 298 },
        ]),
        new JsCurve([
            { x: 300, y: 214 },
            { x: 264, y: 344 },
            { x: 227, y: 387 },
        ]),
    ]);

    const [highlightLeash, setHighlightLeash] = useState<
        [number, number] | null
    >(null);

    return (
        <div style={{ display: 'flex' }}>
            <CurveSpaceView
                curves={curves}
                updateCurves={setCurves}
                highlightLeash={highlightLeash}
            />
            <ParamSpaceView
                curves={curves}
                highlightLeash={highlightLeash}
                setHighlightLeash={setHighlightLeash}
            />
        </div>
    );
}
