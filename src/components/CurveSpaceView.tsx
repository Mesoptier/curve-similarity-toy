import { type Dispatch, Fragment, type SetStateAction } from 'react';

import { JsCurve } from '@rs_lib';
import { CURVE_COLORS } from '../curves';

function makePathDefinition(curve: JsCurve): string {
    const { points } = curve;
    if (points.length === 0) {
        return '';
    }
    return 'M' + points.map(({ x, y }) => `${x},${y}`).join(' ');
}

interface CurveSpaceViewProps {
    curves: [JsCurve, JsCurve];
    updateCurves: Dispatch<SetStateAction<JsCurve[]>>;
}

export function CurveSpaceView(props: CurveSpaceViewProps): JSX.Element {
    const { curves, updateCurves } = props;

    return (
        <svg
            width={500}
            height={500}
            onClick={(e) => {
                const curveIdx = e.ctrlKey ? 1 : 0;
                const newPoint = {
                    x: e.clientX - e.currentTarget.getBoundingClientRect().x,
                    y: e.clientY - e.currentTarget.getBoundingClientRect().y,
                };

                updateCurves((curves) => {
                    curves = [...curves];
                    curves[curveIdx] = curves[curveIdx].with_point(newPoint);
                    return curves;
                });
            }}
            style={{ border: '1px solid gray' }}
        >
            {curves.map((curve, curveIdx) => (
                <Fragment key={curveIdx}>
                    {curve.points.map(({ x, y }, pointIdx) => (
                        <circle
                            key={pointIdx}
                            cx={x}
                            cy={y}
                            r={5}
                            fill={CURVE_COLORS[curveIdx]}
                        />
                    ))}
                    <path
                        d={makePathDefinition(curve)}
                        stroke={CURVE_COLORS[curveIdx]}
                        strokeWidth={2}
                        fill="none"
                    />
                </Fragment>
            ))}
        </svg>
    );
}
