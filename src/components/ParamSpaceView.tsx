import {
    type Dispatch,
    type SetStateAction,
    useEffect,
    useMemo,
    useRef,
    useState,
} from 'react';

import { JsCurve, Plotter } from '@rs_lib';
import { useDrag } from '@use-gesture/react';

import { useBoundingClientRect } from '../hooks/useBoundingClientRect';

const CURVE_OFFSET = 20;
const PLOT_OFFSET = 40;
const OVERFLOW_OFFSET = 20;

interface ParamSpaceViewCanvasProps {
    width: number;
    height: number;
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
    const xMin = Math.round(xCoords[0]);
    const xMax = Math.round(xCoords[xCoords.length - 1]);
    const yMin = Math.round(yCoords[0]);
    const yMax = Math.round(yCoords[yCoords.length - 1]);

    const verticalLines = xCoords
        .filter((_, index, array) => index !== 0 && index !== array.length - 1)
        .map((x) => Math.round(x - 0.5) + 0.5)
        .map((x) => `M${x} ${yMin}V${yMax}`)
        .join('');
    const horizontalLines = yCoords
        .filter((_, index, array) => index !== 0 && index !== array.length - 1)
        .map((y) => Math.round(y - 0.5) + 0.5)
        .map((y) => `M${xMin} ${y}H${xMax}`)
        .join('');
    return verticalLines + horizontalLines;
}

export function ParamSpaceView(props: ParamSpaceViewProps): JSX.Element {
    const { curves, ...otherProps } = props;

    const [showMesh, setShowMesh] = useState(false);

    const [containerElement, setContainerElement] =
        useState<HTMLElement | null>(null);
    const containerRect = useBoundingClientRect(containerElement);

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
            <div ref={setContainerElement} className="space-view__canvas">
                {containerRect && curves.every((curve) => curve.points.length > 1) && (
                    <ParamSpaceViewCanvas
                        width={containerRect.width}
                        height={containerRect.height}
                        curves={curves}
                        showMesh={showMesh}
                        {...otherProps}
                    />
                )}
            </div>
        </div>
    );
}

type Bounds = [min: number, max: number];

function useComputeBounds(
    cumulativeLengths: number[],
    offset: number,
    maxPlotLength: number,
): Bounds {
    const totalLength = cumulativeLengths[cumulativeLengths.length - 1];
    const min = Math.max(0, 0 - offset);
    const max = Math.min(totalLength, maxPlotLength - offset);
    const bounds: Bounds = [min, max];
    return useMemo(() => bounds, bounds);
}

function ParamSpaceViewCanvas(props: ParamSpaceViewCanvasProps): JSX.Element {
    const {
        width,
        height,
        showMesh,
        curves,
        highlightLeash,
        setHighlightLeash,
    } = props;

    const [cumulativeLengths1, cumulativeLengths2] = curves.map(
        (curve) => curve.cumulative_lengths,
    );

    const maxPlotWidth = Math.max(
        0,
        width - (PLOT_OFFSET + CURVE_OFFSET),
    );
    const maxPlotHeight = Math.max(
        0,
        height - (PLOT_OFFSET + CURVE_OFFSET),
    );

    const [isDragging, setDragging] = useState(false);
    const [[xOffset, yOffset], setTranslation] = useState([0, 0]);

    const targetRef = useRef(null);

    useDrag(
        (state) => {
            const [xOffset, yOffset] = state.offset;
            setTranslation([xOffset, -yOffset]);
            setDragging(state.dragging);

            if (state.dragging) {
                setHighlightLeash(null);
            }
        },
        { target: targetRef },
    );

    const xBounds = useComputeBounds(cumulativeLengths1, xOffset, maxPlotWidth);
    const yBounds = useComputeBounds(
        cumulativeLengths2,
        yOffset,
        maxPlotHeight,
    );

    const plotWidth = xBounds[1] - xBounds[0];
    const plotHeight = yBounds[1] - yBounds[0];

    return (
        <svg ref={targetRef} style={{ touchAction: 'none' }}>
            <g transform={`translate(0, ${height}) scale(1, -1)`}>
                {/* Flattened curves along axes */}
                <g
                    className="curve"
                    data-curve-idx={0}
                    transform={`translate(${PLOT_OFFSET}, ${CURVE_OFFSET})`}
                >
                    <CurveAxis
                        cumulativeLengths={cumulativeLengths1}
                        offset={xOffset}
                        maxLength={maxPlotWidth}
                    />
                </g>
                <g
                    className="curve"
                    data-curve-idx={1}
                    transform={`translate(${CURVE_OFFSET}, ${PLOT_OFFSET}) rotate(90)`}
                >
                    <CurveAxis
                        cumulativeLengths={cumulativeLengths2}
                        offset={yOffset}
                        maxLength={maxPlotHeight}
                    />
                </g>

                {/* Plot canvas */}
                <foreignObject
                    className="plot-canvas"
                    x={PLOT_OFFSET + Math.max(0, xOffset)}
                    y={PLOT_OFFSET + Math.max(0, yOffset)}
                    width={Math.round(plotWidth)}
                    height={Math.round(plotHeight)}
                    onMouseMove={(e) => {
                        if (isDragging) {
                            return;
                        }

                        const { x, y, height } =
                            e.currentTarget.getBoundingClientRect();

                        setHighlightLeash([
                            e.clientX - x - Math.min(0, xOffset),
                            height - (e.clientY - y) - Math.min(0, yOffset),
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
                        xBounds={xBounds}
                        yBounds={yBounds}
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
                            cumulativeLengths1.map(
                                (length) =>
                                    Math.max(
                                        xBounds[0],
                                        Math.min(xBounds[1], length),
                                    ) + xOffset,
                            ),
                            cumulativeLengths2.map(
                                (length) =>
                                    Math.max(
                                        yBounds[0],
                                        Math.min(yBounds[1], length),
                                    ) + yOffset,
                            ),
                        )}
                    />
                    {!isDragging && highlightLeash && (
                        <g className="leash">
                            <line
                                className="leash__line leash__line--dashed"
                                x1={-(PLOT_OFFSET - CURVE_OFFSET)}
                                y1={highlightLeash[1] + yOffset}
                                x2={highlightLeash[0] + xOffset}
                                y2={highlightLeash[1] + yOffset}
                            />
                            <line
                                className="leash__line leash__line--dashed"
                                x1={highlightLeash[0] + xOffset}
                                y1={-(PLOT_OFFSET - CURVE_OFFSET)}
                                x2={highlightLeash[0] + xOffset}
                                y2={highlightLeash[1] + yOffset}
                            />
                            <circle
                                className="leash__point"
                                cx={highlightLeash[0] + xOffset}
                                cy={highlightLeash[1] + yOffset}
                            />
                            <circle
                                className="leash__point"
                                cx={-(PLOT_OFFSET - CURVE_OFFSET)}
                                cy={highlightLeash[1] + yOffset}
                            />
                            <circle
                                className="leash__point"
                                cx={highlightLeash[0] + xOffset}
                                cy={-(PLOT_OFFSET - CURVE_OFFSET)}
                            />
                        </g>
                    )}
                </g>
            </g>
        </svg>
    );
}

interface CurveAxisProps {
    cumulativeLengths: number[];
    offset: number;
    maxLength: number;
}

function CurveAxis(props: CurveAxisProps): JSX.Element {
    const { cumulativeLengths, offset, maxLength } = props;
    const totalLength = cumulativeLengths[cumulativeLengths.length - 1];

    return (
        <>
            <line
                className="curve__line curve__line--overflow"
                x1={Math.max(offset, 0)}
                x2={Math.max(offset, 0 - OVERFLOW_OFFSET)}
                y1={0}
                y2={0}
            />
            <line
                className="curve__line"
                x1={Math.max(offset, 0)}
                x2={Math.min(offset + totalLength, maxLength)}
                y1={0}
                y2={0}
            />
            <line
                className="curve__line curve__line--overflow"
                x1={Math.min(offset + totalLength, maxLength)}
                x2={Math.min(offset + totalLength, maxLength + OVERFLOW_OFFSET)}
                y1={0}
                y2={0}
            />
            {cumulativeLengths
                .map((length) => offset + length)
                .filter(
                    (length) =>
                        0 - OVERFLOW_OFFSET <= length &&
                        length <= maxLength + OVERFLOW_OFFSET,
                )
                .map((length, idx) => {
                    let scale = 1;
                    if (length <= 0) {
                        scale = 1 - (0 - length) / OVERFLOW_OFFSET;
                    } else if (maxLength <= length) {
                        scale = 1 - (length - maxLength) / OVERFLOW_OFFSET;
                    }
                    return (
                        <circle
                            key={idx}
                            className="curve__point"
                            cx={length}
                            cy={0}
                            style={{ '--scale': scale }}
                        />
                    );
                })}
        </>
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
    xBounds: Bounds;
    yBounds: Bounds;
    showMesh: boolean;
}

function Plot(props: PlotProps): JSX.Element {
    const { curves, height, width, xBounds, yBounds, showMesh } = props;

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
            x_bounds: xBounds,
            y_bounds: yBounds,
            canvas_width: canvasWidth,
            canvas_height: canvasHeight,
            device_pixel_ratio: devicePixelRatio,
        });
    }, [
        plotter,
        curves,
        showMesh,
        xBounds,
        yBounds,
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
