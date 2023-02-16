import { useState } from 'react';

import { JsCurve } from '@rs_lib';
import { CurveSpaceView } from './CurveSpaceView';
import { ParamSpaceView } from './ParamSpaceView';

export function App(): JSX.Element {
    const [curves, setCurves] = useState<[JsCurve, JsCurve]>([
        new JsCurve([
            [1.98, 1.58],
            [1.50, 2.48],
            [0.91, 2.98],
        ]),
        new JsCurve([
            [3.00, 2.14],
            [2.64, 3.44],
            [2.27, 3.87],
        ]),
    ]);

    const [highlightLeash, setHighlightLeash] = useState<
        [number, number] | null
    >(null);

    return (
        <div className="space-view-container">
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
