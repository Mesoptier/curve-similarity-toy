import { useEffect, useState } from 'react';

import { Plotter, JsCurve } from '../../rs_lib/pkg';
import { CURVE_COLORS } from '../curves';

interface ParamSpaceViewProps {
    curves: [JsCurve, JsCurve];
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
