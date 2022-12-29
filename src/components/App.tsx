import { useState } from 'react';

import { CurveSpaceView } from './CurveSpaceView';
import { type Curve } from '../curves';
import { ParamSpaceView } from './ParamSpaceView';

export function App(): JSX.Element {
    const [curves, setCurves] = useState<[Curve, Curve]>([
        [
            { x: 198, y: 158 },
            { x: 150, y: 248 },
            { x: 91, y: 298 },
        ],
        [
            { x: 227, y: 387 },
            { x: 264, y: 344 },
            { x: 300, y: 214 },
        ],
    ]);

    return (
        <div style={{ display: 'flex' }}>
            <CurveSpaceView curves={curves} updateCurves={setCurves} />
            <ParamSpaceView curves={curves} />
        </div>
    );
}
