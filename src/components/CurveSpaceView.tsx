import { Fragment, useState } from 'react';

type Point = { x: number; y: number };
type Curve = Point[];

const CURVE_COLORS = ['red', 'yellow'];

function makePathDefinition(curve: Curve): string {
    if (curve.length === 0) {
        return '';
    }

    return 'M' + curve.map(({ x, y }) => `${x},${y}`).join(' ');
}

export function CurveSpaceView(): JSX.Element {
    const [curves, setCurves] = useState<Curve[]>([[], []]);

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

                setCurves((curves) => {
                    curves = [...curves];
                    curves[curveIdx].push(newPoint);
                    return curves;
                });
            }}
            style={{ border: '1px solid gray' }}
        >
            {curves.map((curve, curveIdx) => (
                <Fragment key={curveIdx}>
                    {curve.map(({ x, y }, pointIdx) => (
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
