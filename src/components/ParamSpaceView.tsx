import { type Dispatch, type SetStateAction, useEffect, useState } from 'react';

import { JsCurve, Plotter } from '@rs_lib';

const CURVE_OFFSET = 20;
const PLOT_OFFSET = 40;

interface ParamSpaceViewCanvasProps {
    containerSize: { width: number; height: number };
    showMesh: boolean;

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

    const [showMesh, setShowMesh] = useState(false);

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
                <label className="space-view__tool">
                    <input
                        type="checkbox"
                        checked={showMesh}
                        onChange={(e) => setShowMesh(e.target.checked)}
                    />
                    Show mesh
                </label>
            </header>
            <div ref={setContainer} className="space-view__canvas">
                {curves.every((curve) => curve.points.length > 1) && (
                    <ParamSpaceViewCanvas
                        containerSize={containerSize}
                        curves={curves}
                        showMesh={showMesh}
                        {...otherProps}
                    />
                )}
            </div>
        </div>
    );
}

function ParamSpaceViewCanvas(props: ParamSpaceViewCanvasProps): JSX.Element {
    const {
        containerSize,
        showMesh,
        curves,
        highlightLeash,
        setHighlightLeash,
    } = props;

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
                        showMesh={showMesh}
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

function useDevicePixelRatio(): number {
    const [devicePixelRatio, setDevicePixelRatio] = useState(
        window.devicePixelRatio,
    );
    useEffect(() => {
        const media = window.matchMedia(
            `(resolution: ${devicePixelRatio}dppx)`,
        );
        const handleChange = () => {
            setDevicePixelRatio(window.devicePixelRatio);
        };

        media.addEventListener('change', handleChange);
        return () => {
            media.removeEventListener('change', handleChange);
        };
    }, [devicePixelRatio]);
    return devicePixelRatio;
}

interface PlotProps {
    curves: [JsCurve, JsCurve];
    width: number;
    height: number;
    showMesh: boolean;
}

function Plot(props: PlotProps): JSX.Element {
    const { curves, height, width, showMesh } = props;

    const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
    const [plotter, setPlotter] = useState<Plotter | null>(null);

    const devicePixelRatio = useDevicePixelRatio();
    const canvasWidth = Math.round(width * devicePixelRatio);
    const canvasHeight = Math.round(height * devicePixelRatio);

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
        plotter.draw({
            show_mesh: showMesh,
            x_bounds: [0, width],
            y_bounds: [0, height],
            canvas_width: canvasWidth,
            canvas_height: canvasHeight,
            device_pixel_ratio: devicePixelRatio,
        });
    }, [
        plotter,
        curves,
        showMesh,
        width,
        height,
        canvasWidth,
        canvasHeight,
        devicePixelRatio,
    ]);

    return (
        <canvas
            ref={setCanvas}
            width={canvasWidth}
            height={canvasHeight}
            style={{
                width: `${width}px`,
                height: `${height}px`,
                transform: 'scale(1, -1)',
                transformOrigin: '50% 50%',
            }}
        />
    );
}
