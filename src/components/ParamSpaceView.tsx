import { type Dispatch, type SetStateAction, useEffect, useState } from 'react';

import { Plotter, JsCurve } from '@rs_lib';
import { CURVE_COLORS } from '../curves';

interface ParamSpaceViewProps {
    curves: [JsCurve, JsCurve];
    highlightLeash: [number, number] | null;
    setHighlightLeash: Dispatch<SetStateAction<[number, number] | null>>;
}

function makeGridLines(xCoords: number[], yCoords: number[]): string {
    const verticalLines = xCoords
        .map((x) => `M${x} ${yCoords[0]}V${yCoords[yCoords.length - 1]}`)
        .join('');
    const horizontalLines = yCoords
        .map((y) => `M${xCoords[0]} ${y}H${xCoords[xCoords.length - 1]}`)
        .join('');

    return verticalLines + horizontalLines;
}

export function ParamSpaceView(props: ParamSpaceViewProps): JSX.Element {
    const { curves, highlightLeash, setHighlightLeash } = props;

    if (curves.some((curve) => curve.points.length <= 1)) {
        return null;
    }

    const [cumulativeLengths1, cumulativeLengths2] = curves.map(
        (curve) => curve.cumulative_lengths,
    );

    const width = cumulativeLengths1[cumulativeLengths1.length - 1];
    const height = cumulativeLengths2[cumulativeLengths2.length - 1];

    return (
        <svg
            width={width + 30}
            height={height + 30}
            style={{ border: '1px solid gray' }}
        >
            <g transform={`translate(20, ${height + 20})`}>
                <line
                    x1={0}
                    y1={0}
                    x2={cumulativeLengths1[cumulativeLengths1.length - 1]}
                    y2={0}
                    stroke={CURVE_COLORS[0]}
                    strokeWidth={2}
                />
                {cumulativeLengths1.map((x, pointIdx) => (
                    <circle
                        key={pointIdx}
                        cx={x}
                        cy={0}
                        r={5}
                        fill={CURVE_COLORS[0]}
                    />
                ))}
            </g>
            <g transform={`translate(10, ${height + 30 - 20}) scale(1, -1)`}>
                <line
                    x1={0}
                    y1={0}
                    x2={0}
                    y2={cumulativeLengths2[cumulativeLengths2.length - 1]}
                    stroke={CURVE_COLORS[1]}
                    strokeWidth={2}
                />
                {cumulativeLengths2.map((y, pointIdx) => (
                    <circle
                        key={pointIdx}
                        cx={0}
                        cy={y}
                        r={5}
                        fill={CURVE_COLORS[1]}
                    />
                ))}
            </g>
            <foreignObject
                x={20}
                y={10}
                width={width}
                height={height}
                onMouseMove={(e) => {
                    const { x, y, height } =
                        e.currentTarget.getBoundingClientRect();

                    setHighlightLeash([
                        e.clientX - x,
                        height - (e.clientY - y),
                    ]);
                }}
                onMouseLeave={() => {
                    setHighlightLeash(null);
                }}
            >
                <Plot curves={curves} width={width} height={height} />
            </foreignObject>
            <g
                transform={`translate(20, ${height + 30 - 20}) scale(1, -1)`}
                style={{ pointerEvents: 'none' }}
            >
                <path
                    d={makeGridLines(cumulativeLengths1, cumulativeLengths2)}
                    fill="none"
                    stroke="white"
                />
                {highlightLeash && (
                    <>
                        <line
                            x2={highlightLeash[0]}
                            y2={highlightLeash[1]}
                            x1={-10}
                            y1={highlightLeash[1]}
                            stroke="green"
                            strokeWidth={2}
                            strokeDasharray="2 4"
                            strokeLinecap="round"
                        />
                        <line
                            x2={highlightLeash[0]}
                            y2={highlightLeash[1]}
                            x1={highlightLeash[0]}
                            y1={-10}
                            stroke="green"
                            strokeWidth={2}
                            strokeDasharray="2 4"
                            strokeLinecap="round"
                        />
                        <circle
                            cx={highlightLeash[0]}
                            cy={highlightLeash[1]}
                            r={3}
                            fill="green"
                        />
                        <circle
                            cx={-10}
                            cy={highlightLeash[1]}
                            r={3}
                            fill="green"
                        />
                        <circle
                            cx={highlightLeash[0]}
                            cy={-10}
                            r={3}
                            fill="green"
                        />
                    </>
                )}
            </g>
        </svg>
    );
}

interface PlotProps {
    curves: [JsCurve, JsCurve];
    width: number;
    height: number;
}

function Plot(props: PlotProps): JSX.Element {
    const { curves, height, width } = props;

    const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
    const [plotter, setPlotter] = useState<Plotter | null>(null);

    useEffect(() => {
        if (canvas === null) {
            return;
        }

        const ctx = canvas.getContext('webgl2');
        setPlotter(new Plotter(ctx));
    }, [canvas]);

    useEffect(() => {
        if (plotter === null) {
            return;
        }

        plotter.update_curves(...curves);
        plotter.resize(width, height);
        plotter.draw();
    }, [plotter, curves, width, height]);

    return <canvas ref={setCanvas} width={width} height={height} />;
}
