import { type Dispatch, type SetStateAction, useEffect, useState } from 'react';

import { Plotter, JsCurve } from '@rs_lib';

const CURVE_OFFSET = 20;
const PLOT_OFFSET = 40;

interface ParamSpaceViewCanvasProps {
    containerSize: { width: number; height: number };

    curves: [JsCurve, JsCurve];
    highlightLeash: [number, number] | null;
    setHighlightLeash: Dispatch<SetStateAction<[number, number] | null>>;
}

type ParamSpaceViewProps = Pick<
    ParamSpaceViewCanvasProps,
    'curves' | 'highlightLeash' | 'setHighlightLeash'
>;

function makeGridLines(xCoords: number[], yCoords: number[]): string {
    const verticalLines = xCoords
        .filter((_, index, array) => index !== 0 && index !== array.length - 1)
        .map((x) => Math.round(x - 0.5) + 0.5)
        .map((x) => `M${x} 0V${Math.round(yCoords[yCoords.length - 1])}`)
        .join('');
    const horizontalLines = yCoords
        .filter((_, index, array) => index !== 0 && index !== array.length - 1)
        .map((y) => Math.round(y - 0.5) + 0.5)
        .map((y) => `M0 ${y}H${Math.round(xCoords[xCoords.length - 1])}`)
        .join('');
    return verticalLines + horizontalLines;
}

export function ParamSpaceView(props: ParamSpaceViewProps): JSX.Element {
    const { curves, ...otherProps } = props;

    const [container, setContainer] = useState<HTMLElement | null>(null);
    const [containerSize, setContainerSize] = useState<{
        width: number;
        height: number;
    }>({ width: 0, height: 0 });

    useEffect(() => {
        if (container === null) {
            return;
        }

        const observer = new ResizeObserver((entries) => {
            const { width, height } = entries[0].contentRect;
            setContainerSize({ width, height });
        });
        observer.observe(container, { box: 'border-box' });
        return () => {
            observer.disconnect();
        };
    }, [container]);

    return (
        <div className="space-view">
            <header className="space-view__header">
                <div className="space-view__title">Parameter space</div>
            </header>
            <div ref={setContainer} className="space-view__canvas">
                {curves.every((curve) => curve.points.length > 1) && (
                    <ParamSpaceViewCanvas
                        containerSize={containerSize}
                        curves={curves}
                        {...otherProps}
                    />
                )}
            </div>
        </div>
    );
}

function ParamSpaceViewCanvas(props: ParamSpaceViewCanvasProps): JSX.Element {
    const { containerSize, curves, highlightLeash, setHighlightLeash } = props;

    const [cumulativeLengths1, cumulativeLengths2] = curves.map(
        (curve) => curve.cumulative_lengths,
    );

    const plotWidth = cumulativeLengths1[cumulativeLengths1.length - 1];
    const plotHeight = cumulativeLengths2[cumulativeLengths2.length - 1];

    return (
        <svg>
            <g transform={`translate(0, ${containerSize.height}) scale(1, -1)`}>
                {/* Flattened curves along axes */}
                {[
                    cumulativeLengths1.map((x) => ({ x, y: 0 })),
                    cumulativeLengths2.map((y) => ({ x: 0, y })),
                ].map((coords, curveIdx) => (
                    <g
                        key={curveIdx}
                        className="curve"
                        data-curve-idx={curveIdx}
                        transform={
                            curveIdx === 0
                                ? `translate(${PLOT_OFFSET}, ${CURVE_OFFSET})`
                                : `translate(${CURVE_OFFSET}, ${PLOT_OFFSET})`
                        }
                    >
                        <line
                            className="curve__line"
                            x1={0}
                            y1={0}
                            x2={coords[coords.length - 1].x}
                            y2={coords[coords.length - 1].y}
                        />
                        {coords.map(({ x, y }, pointIdx) => (
                            <circle
                                key={pointIdx}
                                className="curve__point"
                                cx={x}
                                cy={y}
                            />
                        ))}
                    </g>
                ))}

                {/* Plot canvas */}
                <foreignObject
                    className="plot-canvas"
                    x={PLOT_OFFSET}
                    y={PLOT_OFFSET}
                    width={Math.round(plotWidth)}
                    height={Math.round(plotHeight)}
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
                    <Plot
                        curves={curves}
                        width={Math.round(plotWidth)}
                        height={Math.round(plotHeight)}
                    />
                </foreignObject>

                {/* Plot overlay */}
                <g
                    className="plot-overlay"
                    transform={`translate(${PLOT_OFFSET}, ${PLOT_OFFSET})`}
                >
                    <path
                        className="grid-lines"
                        d={makeGridLines(
                            cumulativeLengths1,
                            cumulativeLengths2,
                        )}
                    />
                    {highlightLeash && (
                        <g className="leash">
                            <line
                                className="leash__line leash__line--dashed"
                                x1={-(PLOT_OFFSET - CURVE_OFFSET)}
                                y1={highlightLeash[1]}
                                x2={highlightLeash[0]}
                                y2={highlightLeash[1]}
                            />
                            <line
                                className="leash__line leash__line--dashed"
                                x1={highlightLeash[0]}
                                y1={-(PLOT_OFFSET - CURVE_OFFSET)}
                                x2={highlightLeash[0]}
                                y2={highlightLeash[1]}
                            />
                            <circle
                                className="leash__point"
                                cx={highlightLeash[0]}
                                cy={highlightLeash[1]}
                            />
                            <circle
                                className="leash__point"
                                cx={-(PLOT_OFFSET - CURVE_OFFSET)}
                                cy={highlightLeash[1]}
                            />
                            <circle
                                className="leash__point"
                                cx={highlightLeash[0]}
                                cy={-(PLOT_OFFSET - CURVE_OFFSET)}
                            />
                        </g>
                    )}
                </g>
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

        const ctx = canvas.getContext('webgl2', { alpha: false });
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

    return (
        <canvas
            ref={setCanvas}
            width={width}
            height={height}
            style={{ transform: 'scale(1, -1)', transformOrigin: '50% 50%' }}
        />
    );
}
