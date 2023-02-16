import {
    type Dispatch,
    type SetStateAction,
    useEffect,
    useLayoutEffect,
    useMemo,
    useRef,
    useState,
} from 'react';
import {
    Coordinates,
    Mafs,
    usePaneContext,
    useTransformContext,
    vec,
} from 'mafs';

import { ILengths, JsCurve, Plotter } from '@rs_lib';

import { useBoundingClientRect } from '../hooks/useBoundingClientRect';

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
                {containerRect &&
                    curves.every((curve) => curve.points.length > 1) && (
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

function ParamSpaceViewCanvas(props: ParamSpaceViewCanvasProps): JSX.Element {
    const { width, height, curves, showMesh } = props;

    const cumulativeLengths = curves.map(
        (curve) => curve.cumulative_lengths,
    ) as [ILengths, ILengths];
    const totalLengths = cumulativeLengths.map(
        (lengths) => lengths[lengths.length - 1],
    ) as [number, number];

    return (
        <Mafs width={width} height={height} zoom>
            <Coordinates.Cartesian />
            <HeightPlot
                curves={curves}
                totalLengths={totalLengths}
                showMesh={showMesh}
            />
        </Mafs>
    );
}

interface HeightPlotProps {
    curves: [JsCurve, JsCurve];
    totalLengths: [number, number];
    showMesh: boolean;
}

function HeightPlot(props: HeightPlotProps) {
    const { curves, totalLengths, showMesh } = props;

    const { userTransform, viewTransform } = useTransformContext();

    const scaleX = Math.abs(viewTransform[0]) * Math.abs(userTransform[0]);
    const scaleY = Math.abs(viewTransform[4]) * Math.abs(userTransform[4]);

    const { xPaneRange, yPaneRange } = usePaneContext();
    const xRangeUnmemoized = [
        Math.max(0, xPaneRange[0]),
        Math.min(totalLengths[0], xPaneRange[1]),
    ] as Bounds;
    const yRangeUnmemoized = [
        Math.max(0, yPaneRange[0]),
        Math.min(totalLengths[1], yPaneRange[1]),
    ] as Bounds;

    const xRange = useMemo(() => xRangeUnmemoized, xRangeUnmemoized);
    const yRange = useMemo(() => yRangeUnmemoized, yRangeUnmemoized);

    const drawRange = [
        [xRange[0], yRange[1]] as vec.Vector2,
        [xRange[1], yRange[0]] as vec.Vector2,
    ].map((point) =>
        vec.transform(vec.transform(point, userTransform), viewTransform),
    );
    const canvasRange = [
        [xRange[0], yPaneRange[1]] as vec.Vector2,
        [xPaneRange[1], yRange[0]] as vec.Vector2,
    ].map((point) =>
        vec.transform(vec.transform(point, userTransform), viewTransform),
    );

    const drawX = drawRange[0][0];
    const drawY = drawRange[0][1];
    const drawWidth = drawRange[1][0] - drawRange[0][0];
    const drawHeight = drawRange[1][1] - drawRange[0][1];
    const canvasWidth = canvasRange[1][0] - canvasRange[0][0];
    const canvasHeight = canvasRange[1][1] - canvasRange[0][1];

    return (
        <HeightPlotCanvas
            curves={curves}
            drawX={drawX}
            drawY={drawY}
            drawWidth={drawWidth}
            drawHeight={drawHeight}
            canvasWidth={canvasWidth}
            canvasHeight={canvasHeight}
            xBounds={xRange}
            yBounds={yRange}
            scaleX={scaleX}
            scaleY={scaleY}
            showMesh={showMesh}
        />
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

interface HeightPlotCanvasProps {
    curves: [JsCurve, JsCurve];
    drawX: number;
    drawY: number;
    drawWidth: number;
    drawHeight: number;
    canvasWidth: number;
    canvasHeight: number;
    xBounds: Bounds;
    yBounds: Bounds;
    scaleX: number;
    scaleY: number;
    showMesh: boolean;
}

function HeightPlotCanvas(props: HeightPlotCanvasProps): JSX.Element {
    const {
        curves,
        drawX,
        drawY,
        drawWidth,
        drawHeight,
        canvasWidth,
        canvasHeight,
        xBounds,
        yBounds,
        scaleX,
        scaleY,
        showMesh,
    } = props;

    const foreignObject = useRef<SVGForeignObjectElement>(null);
    const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
    const [plotter, setPlotter] = useState<Plotter | null>(null);

    const devicePixelRatio = useDevicePixelRatio();
    const devicePixelRound = (x) =>
        Math.round(x * devicePixelRatio) / devicePixelRatio;

    useEffect(() => {
        if (canvas === null) {
            return;
        }

        const ctx = canvas.getContext('webgl2', { alpha: false });
        setPlotter(new Plotter(ctx));
    }, [canvas]);

    useLayoutEffect(() => {
        if (plotter === null) {
            return;
        }

        plotter.update_curves(...curves);
        plotter.draw({
            show_mesh: showMesh,
            x_bounds: xBounds,
            y_bounds: yBounds,
            x_scale: scaleX,
            y_scale: scaleY,
            draw_width: Math.round(drawWidth * devicePixelRatio),
            draw_height: Math.round(drawHeight * devicePixelRatio),
            canvas_width: Math.round(canvasWidth * devicePixelRatio),
            canvas_height: Math.round(canvasHeight * devicePixelRatio),
            device_pixel_ratio: devicePixelRatio,
        });
    }, [
        plotter,
        curves,
        showMesh,
        xBounds,
        yBounds,
        scaleX,
        scaleY,
        drawWidth,
        drawHeight,
        canvasWidth,
        canvasHeight,
        devicePixelRatio,
    ]);

    useLayoutEffect(() => {
        // Transform the foreignObject such that it aligns perfectly to the
        // device pixel grid.
        const rect = foreignObject.current.closest('svg').viewBox.animVal;
        const [offsetX, offsetY] = !rect
            ? [0, 0]
            : [
                  rect.x - devicePixelRound(rect.x),
                  rect.y - devicePixelRound(rect.y),
              ];
        foreignObject.current.setAttribute(
            'transform',
            `translate(${offsetX}, ${offsetY})`,
        );
    });

    return (
        <foreignObject
            ref={foreignObject}
            x={devicePixelRound(drawX)}
            y={devicePixelRound(drawY)}
            width={devicePixelRound(drawWidth)}
            height={devicePixelRound(drawHeight)}
        >
            <canvas
                ref={setCanvas}
                width={Math.round(canvasWidth * devicePixelRatio)}
                height={Math.round(canvasHeight * devicePixelRatio)}
                style={{
                    width: devicePixelRound(canvasWidth),
                    height: devicePixelRound(canvasHeight),
                }}
            />
        </foreignObject>
    );
}
