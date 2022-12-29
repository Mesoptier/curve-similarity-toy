import { type Curve, CURVE_COLORS, type Point } from '../curves';
import { useEffect, useState } from 'react';

import init, { Plotter } from '../../rs_lib/pkg';
// @ts-ignore: esbuild is configured to export the filename of the .wasm file
import wasmFilePath from '../../rs_lib/pkg/rs_lib_bg.wasm';

interface ParamSpaceViewProps {
    curves: [Curve, Curve];
}

/**
 * Computes Euclidean distance two points.
 */
function dist(p1: Point, p2: Point): number {
    return Math.sqrt((p1.x - p2.x) ** 2 + (p1.y - p2.y) ** 2);
}

/**
 * Computes the cumulative arc length of the curve up to each point.
 */
function computeCumulativeLengths(curve: Curve): number[] {
    let cumulativeLength = 0;
    return curve.map((point, pointIdx) => {
        if (pointIdx !== 0) {
            cumulativeLength += dist(point, curve[pointIdx - 1]);
        }
        return cumulativeLength;
    });
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
    const { curves } = props;

    if (curves.some((curve) => curve.length <= 1)) {
        return null;
    }

    const [cumulativeLengths1, cumulativeLengths2] = curves.map(
        computeCumulativeLengths,
    );

    const width = cumulativeLengths1[cumulativeLengths1.length - 1];
    const height = cumulativeLengths2[cumulativeLengths2.length - 1];

    return (
        <svg
            width={width + 30}
            height={height + 30}
            style={{ border: '1px solid gray' }}
        >
            {cumulativeLengths1.map((x, pointIdx) => (
                <circle
                    key={pointIdx}
                    cx={x + 20}
                    cy={height + 30 - 10}
                    r={5}
                    fill={CURVE_COLORS[0]}
                />
            ))}
            {cumulativeLengths2.map((y, pointIdx) => (
                <circle
                    key={pointIdx}
                    cx={10}
                    cy={height + 30 - (y + 20)}
                    r={5}
                    fill={CURVE_COLORS[1]}
                />
            ))}
            <foreignObject x={20} y={10} width={width} height={height}>
                <Plot curves={curves} width={width} height={height} />
            </foreignObject>
            <path
                transform={`translate(20, ${height + 30 - 20}) scale(1, -1)`}
                d={makeGridLines(cumulativeLengths1, cumulativeLengths2)}
                fill="none"
                stroke="white"
            />
        </svg>
    );
}

interface PlotProps {
    curves: [Curve, Curve];
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

        init(new URL(wasmFilePath, import.meta.url)).then(() => {
            setPlotter(new Plotter(ctx));
        });
    }, [canvas]);

    useEffect(() => {
        plotter?.update_curves(curves);
        plotter?.resize(width, height);
        plotter?.draw();
    }, [plotter, curves, width, height]);

    return <canvas ref={setCanvas} width={width} height={height} />;
}
