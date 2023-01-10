import { type Dispatch, type SetStateAction, useEffect, useState } from 'react';

import { IPoint, JsCurve } from '@rs_lib';

function makePathDefinition(curve: JsCurve): string {
    const { points } = curve;
    if (points.length === 0) {
        return '';
    }
    return 'M' + points.map(({ x, y }) => `${x},${y}`).join(' ');
}

interface CurveSpaceViewCanvasProps {
    curves: [JsCurve, JsCurve];
    updateCurves: Dispatch<SetStateAction<JsCurve[]>>;
    highlightLeash: [number, number] | null;
}

type CurveSpaceViewProps = CurveSpaceViewCanvasProps;

type PreviewPoints = [IPoint | null, IPoint | null];

export function CurveSpaceView(props: CurveSpaceViewProps): JSX.Element {
    return (
        <div className="space-view">
            <header className="space-view__header">
                <div className="space-view__title">Curves space</div>
            </header>
            <CurveSpaceViewCanvas {...props} />
        </div>
    );
}

function CurveSpaceViewCanvas(props: CurveSpaceViewCanvasProps): JSX.Element {
    const { curves, updateCurves, highlightLeash } = props;

    const [previewPoints, setPreviewPoints] = useState<PreviewPoints>([
        null,
        null,
    ]);

    useEffect(() => {
        function handleKeyboardEvent(e: KeyboardEvent) {
            setPreviewPoints((previewPoints) => {
                const curveIdx = e.ctrlKey ? 1 : 0;
                if (previewPoints[curveIdx] === null) {
                    return [previewPoints[1], previewPoints[0]];
                }
                return previewPoints;
            });
        }

        window.addEventListener('keydown', handleKeyboardEvent);
        window.addEventListener('keyup', handleKeyboardEvent);
        return () => {
            window.removeEventListener('keydown', handleKeyboardEvent);
            window.removeEventListener('keyup', handleKeyboardEvent);
        };
    }, []);

    return (
        <svg
            className="space-view__canvas"
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
            onMouseMove={(e) => {
                const curveIdx = e.ctrlKey ? 1 : 0;
                const newPoint = {
                    x: e.clientX - e.currentTarget.getBoundingClientRect().x,
                    y: e.clientY - e.currentTarget.getBoundingClientRect().y,
                };

                const previewPoints: PreviewPoints = [null, null];
                previewPoints[curveIdx] = newPoint;
                setPreviewPoints(previewPoints);
            }}
            onMouseLeave={() => {
                setPreviewPoints([null, null]);
            }}
        >
            {curves.map((curve, curveIdx) => (
                <CurvePreview
                    key={curveIdx}
                    curve={curve}
                    curveIdx={curveIdx}
                    previewPoint={previewPoints[curveIdx]}
                />
            ))}
            {highlightLeash && (
                <LeashPreview curves={curves} leash={highlightLeash} />
            )}
        </svg>
    );
}

interface CurvePreviewProps {
    curve: JsCurve;
    curveIdx: number;
    previewPoint: IPoint | null;
}

function CurvePreview(props: CurvePreviewProps): JSX.Element | null {
    const { curve, curveIdx, previewPoint } = props;

    if (curve.points.length === 0) {
        return null;
    }

    const lastPoint = curve.points[curve.points.length - 1];

    return (
        <g className="curve" data-curve-idx={curveIdx}>
            {curve.points.map(({ x, y }, pointIdx) => (
                <circle key={pointIdx} className="curve__point" cx={x} cy={y} />
            ))}
            <path className="curve__line" d={makePathDefinition(curve)} />
            {previewPoint && (
                <g className="curve__preview">
                    <line
                        className="curve__line"
                        x1={lastPoint.x}
                        y1={lastPoint.y}
                        x2={previewPoint.x}
                        y2={previewPoint.y}
                    />
                    <circle
                        className="curve__point"
                        cx={previewPoint.x}
                        cy={previewPoint.y}
                    />
                </g>
            )}
        </g>
    );
}

interface LeashPreviewProps {
    curves: [JsCurve, JsCurve];
    leash: [number, number];
}

function LeashPreview(props: LeashPreviewProps): JSX.Element {
    const { curves, leash } = props;
    const points = [curves[0].at(leash[0]), curves[1].at(leash[1])];

    return (
        <g className="leash">
            {points.map((point, pointIdx) => (
                <circle
                    className="leash__point"
                    key={pointIdx}
                    cx={point.x}
                    cy={point.y}
                />
            ))}
            <line
                className="leash__line"
                x1={points[0].x}
                y1={points[0].y}
                x2={points[1].x}
                y2={points[1].y}
            />
        </g>
    );
}
