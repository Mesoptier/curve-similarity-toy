import { useState } from 'react';

import { JsCurve } from '../../rs_lib/pkg';
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
            { x: 227, y: 387 },
            { x: 264, y: 344 },
            { x: 300, y: 214 },
        ]),
    ]);

    return (
        <div style={{ display: 'flex' }}>
            <CurveSpaceView curves={curves} updateCurves={setCurves} />
            <ParamSpaceView curves={curves} />
        </div>
    );
}
