import { useState } from 'react';

import { JsCurve } from '@rs_lib';
import { CurveSpaceView } from './CurveSpaceView';
import { ParamSpaceView } from './ParamSpaceView';

export function App(): JSX.Element {
    const [curves, setCurves] = useState<[JsCurve, JsCurve]>([
        new JsCurve([
            [198, 158],
            [150, 248],
            [91, 298],
        ]),
        new JsCurve([
            [300, 214],
            [264, 344],
            [227, 387],
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
