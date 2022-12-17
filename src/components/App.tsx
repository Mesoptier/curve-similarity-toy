import { useState } from 'react';

import { CurveSpaceView } from './CurveSpaceView';
import { type Curve } from '../curves';
import { ParamSpaceView } from './ParamSpaceView';

export function App(): JSX.Element {
    const [curves, setCurves] = useState<[Curve, Curve]>([[], []]);

    return (
        <div style={{ display: 'flex' }}>
            <CurveSpaceView curves={curves} updateCurves={setCurves} />
            <ParamSpaceView curves={curves} />
        </div>
    );
}
